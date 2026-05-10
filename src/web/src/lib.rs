pub mod app;
#[cfg(feature = "sqlite3")]
pub mod worker;

use app::{AppState, AppStateStoreFields};
use fragile::Fragile;
use js_sys::Uint8Array;
use leptos::prelude::*;
use reactive_stores::Store;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Once},
};
use tokio::sync::{OnceCell, mpsc::UnboundedReceiver};
use wasm_bindgen::{JsCast, prelude::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, WorkerError>;

pub struct FragileConfirmed<T> {
    fragile: Fragile<T>,
}

unsafe impl<T> Send for FragileConfirmed<T> {}
unsafe impl<T> Sync for FragileConfirmed<T> {}

impl<T> FragileConfirmed<T> {
    pub fn new(t: T) -> Self {
        FragileConfirmed {
            fragile: Fragile::new(t),
        }
    }
}

impl<T> Deref for FragileConfirmed<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.fragile.get()
    }
}

impl<T> DerefMut for FragileConfirmed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fragile.get_mut()
    }
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum WorkerError {
    #[error(transparent)]
    SQLite(#[from] SQLitendError),
    #[error("DB is not opened")]
    NotOpened,
    #[error("Execute sqlite with invalid state")]
    InvalidState,
    #[error("Failed to load db: {0}")]
    LoadDb(String),
    #[error("Failed to download db: {0}")]
    DownloadDb(String),
    #[error("Unexpected error")]
    Unexpected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadDbResponse {
    pub filename: String,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub data: Uint8Array,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerRequest {
    Open(OpenOptions),
    Run(RunOptions),
    LoadDb(LoadDbOptions),
    DownloadDb,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerResponse {
    Ready,
    Open(Result<()>),
    Run(Result<SQLiteRunResult>),
    LoadDb(Result<()>),
    DownloadDb(Result<DownloadDbResponse>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOptions {
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadDbOptions {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    pub data: Uint8Array,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunOptions {
    pub sql: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InnerError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SQLiteRunResult {
    pub tag: String,
    pub result: Vec<SQLiteStatementResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SQLiteStatementResult {
    Finish,
    Step(SQLiteStatementTable),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SQLiteStatementTable {
    pub sql: String,
    pub values: Option<SQLiteStatementValues>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SQLiteStatementValues {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum SQLitendError {
    #[error("An error occurred while converting a string to a CString")]
    ToCStr,
    #[error("An error occurred while opening the DB: {0:#?}")]
    OpenDb(InnerError),
    #[error("An error occurred while preparing stmt: {0:#?}")]
    Prepare(InnerError),
    #[error("An error occurred while stepping to the next line: {0:#?}")]
    Step(InnerError),
    #[error("An error occurred while getting column name: {0}")]
    GetColumnName(String),
    #[error("The text is not a utf8 string")]
    Utf8Text,
    #[error("The column type is not support: {0}")]
    UnsupportColumnType(i32),
}

pub struct WorkerHandle(Worker);

impl WorkerHandle {
    pub fn send_task(&self, req: WorkerRequest) {
        if let Err(err) = self
            .0
            .post_message(&serde_wasm_bindgen::to_value(&req).unwrap())
        {
            log::error!("Failed to send task to worker: {req:?}, {err:?}");
        }
    }
}

unsafe impl Send for WorkerHandle {}
unsafe impl Sync for WorkerHandle {}

pub fn send_request(state: Store<AppState>, req: WorkerRequest) {
    spawn_local(async move {
        get_worker(state).await.send_task(req);
    });
}

async fn get_worker(state: Store<AppState>) -> &'static WorkerHandle {
    static ONCE: Once = Once::new();
    static WORKER: OnceCell<WorkerHandle> = OnceCell::const_new();

    let worker = WORKER
        .get_or_init(|| async { setup_worker(state, "./sqlite3_loader.js").await })
        .await;

    ONCE.call_once(|| {
        connect_db(state, worker);
    });
    worker
}

fn connect_db(state: Store<AppState>, handle: &'static WorkerHandle) {
    handle.send_task(WorkerRequest::Open(OpenOptions {
        filename: "main.db".into(),
    }));
    let _ = state;
}

async fn setup_worker(state: Store<AppState>, uri: &str) -> WorkerHandle {
    let opts = WorkerOptions::new();
    opts.set_type(WorkerType::Module);

    let worker = match Worker::new_with_options(uri, &opts) {
        Ok(worker) => worker,
        Err(err) => panic!("Failed to setup worker: {err:?}"),
    };

    let notify = Arc::new(tokio::sync::Notify::new());
    let wait = Arc::clone(&notify);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
        match serde_wasm_bindgen::from_value(ev.data()) {
            Ok(WorkerResponse::Ready) => notify.notify_one(),
            Ok(resp) => tx.send(resp).unwrap(),
            Err(err) => log::error!("Failed to parse message {err:?}"),
        }
    });

    spawn_local(handle_responses(state, rx));

    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    wait.notified().await;

    WorkerHandle(worker)
}

async fn handle_responses(state: Store<AppState>, mut rx: UnboundedReceiver<WorkerResponse>) {
    while let Some(resp) = rx.recv().await {
        match resp {
            WorkerResponse::Ready => unreachable!(),
            WorkerResponse::Open(result) => {
                if let Err(err) = result {
                    state.error().set(Some(err.to_string()));
                }
            }
            WorkerResponse::LoadDb(result) => {
                match result {
                    Ok(()) => {
                        state.db_loaded().set(true);
                        state.error().set(None);
                        send_request(
                            state,
                            WorkerRequest::Run(RunOptions {
                                sql: "SELECT name FROM sqlite_master WHERE type='table' OR type='view' ORDER BY name".into(),
                                tag: "tables".into(),
                            }),
                        );
                    }
                    Err(err) => {
                        state.error().set(Some(err.to_string()));
                    }
                }
            }
            WorkerResponse::Run(result) => match result {
                Ok(run_result) => {
                    state.query_error().set(None);
                    handle_run_result(state, run_result);
                }
                Err(err) => {
                    state.query_error().set(Some(err.to_string()));
                }
            },
            WorkerResponse::DownloadDb(result) => match result {
                Ok(resp) => trigger_download(&resp.filename, &resp.data),
                Err(err) => state.error().set(Some(err.to_string())),
            },
        }
    }
}

fn trigger_download(filename: &str, data: &Uint8Array) {
    use leptos::prelude::document;
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, HtmlAnchorElement, Url};

    let array = js_sys::Array::new();
    array.push(data);
    let blob = Blob::new_with_u8_array_sequence(&array).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = document();
    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    Url::revoke_object_url(&url).unwrap();
}

fn handle_run_result(state: Store<AppState>, result: SQLiteRunResult) {
    match result.tag.as_str() {
        "tables" => {
            let mut tables = vec![];
            for item in &result.result {
                if let SQLiteStatementResult::Step(table) = item {
                    if let Some(values) = &table.values {
                        for row in &values.rows {
                            if let Some(name) = row.first() {
                                tables.push(name.clone());
                            }
                        }
                    }
                }
            }
            state.tables().set(tables);
            state.table_columns().set(vec![]);
            state.table_rows().set(vec![]);
        }
        tag if tag.starts_with("data:") => {
            let table_name = tag.strip_prefix("data:").unwrap_or("").to_string();
            for item in &result.result {
                if let SQLiteStatementResult::Step(table) = item {
                    if let Some(values) = &table.values {
                        state.table_columns().set(values.columns.clone());
                        state.table_rows().set(values.rows.clone());
                        state.selected_table().set(Some(table_name));
                        return;
                    }
                }
            }
            state.selected_table().set(Some(table_name));
            state.table_columns().set(vec![]);
            state.table_rows().set(vec![]);
        }
        "query" => {
            state.query_error().set(None);
            let mut found = false;
            for item in &result.result {
                if let SQLiteStatementResult::Step(table) = item {
                    if let Some(values) = &table.values {
                        state.query_columns().set(values.columns.clone());
                        state.query_rows().set(values.rows.clone());
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                state.query_columns().set(vec![]);
                state.query_rows().set(vec![]);
            }
        }
        tag if tag.starts_with("insert:") => {
            let table_name = tag.strip_prefix("insert:").unwrap_or("").to_string();
            send_request(
                state,
                WorkerRequest::Run(RunOptions {
                    sql: format!("SELECT * FROM \"{table_name}\" LIMIT 1000"),
                    tag: format!("data:{table_name}"),
                }),
            );
        }
        _ => {}
    }
}
