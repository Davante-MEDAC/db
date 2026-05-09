mod sqlitend;

use crate::{
    LoadDbOptions, OpenOptions, RunOptions, SQLiteRunResult, WorkerError, WorkerRequest,
    WorkerResponse,
};
use once_cell::sync::Lazy;
use sqlite_wasm_rs::MemVfsUtil;
use sqlite_wasm_rs::WasmOsCallback;
use sqlitend::SQLiteDb;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

type Result<T> = std::result::Result<T, WorkerError>;

struct SyncMemVfs(MemVfsUtil<WasmOsCallback>);
unsafe impl Send for SyncMemVfs {}
unsafe impl Sync for SyncMemVfs {}

static DB: Lazy<Mutex<Option<SQLiteWorker>>> = Lazy::new(|| Mutex::new(None));
static MEM_VFS: Lazy<SyncMemVfs> = Lazy::new(|| SyncMemVfs(MemVfsUtil::new()));

const MEM_VFS_NAME: &str = "memvfs";

fn uri(filename: &str) -> String {
    format!("file:{filename}?vfs={MEM_VFS_NAME}")
}

struct SQLiteWorker {
    open_options: OpenOptions,
    db: Option<Arc<SQLiteDb>>,
}

async fn with_worker<F, T>(mut f: F) -> Result<T>
where
    F: FnMut(&mut SQLiteWorker) -> Result<T>,
{
    f(DB.lock().await.as_mut().ok_or(WorkerError::NotOpened)?)
}

async fn load_db(options: LoadDbOptions) -> Result<()> {
    let db = options.data.to_vec();

    let page_size = sqlite_wasm_rs::utils::check_import_db(&db)
        .map_err(|err| WorkerError::LoadDb(format!("{err}")))?;

    with_worker(|worker| {
        worker.db = None;
        let filename = &worker.open_options.filename;
        MEM_VFS.0.delete_db(filename);
        if let Err(err) = MEM_VFS.0.import_db_unchecked(filename, &db, page_size) {
            return Err(WorkerError::LoadDb(format!("{err}")));
        }
        worker.db = Some(SQLiteDb::open(&uri(filename))?);
        Ok(())
    })
    .await
}

async fn open(options: OpenOptions) -> Result<()> {
    let mut locker = DB.lock().await;
    let filename = options.filename.clone();
    let db = Some(SQLiteDb::open(&uri(&filename))?);
    *locker = Some(SQLiteWorker {
        open_options: options,
        db,
    });
    Ok(())
}

async fn run(options: RunOptions) -> Result<SQLiteRunResult> {
    with_worker(|worker| {
        let db = worker.db.as_ref().ok_or(WorkerError::InvalidState)?;
        let stmts = db.prepare(&options.sql)?;
        let result = stmts.stmts_result()?;
        Ok(SQLiteRunResult {
            tag: options.tag.clone(),
            result,
        })
    })
    .await
}

async fn execute_task(scope: DedicatedWorkerGlobalScope, mut rx: UnboundedReceiver<JsValue>) {
    while let Some(request) = rx.recv().await {
        let request = serde_wasm_bindgen::from_value::<WorkerRequest>(request).unwrap();
        let resp = match request {
            WorkerRequest::Open(options) => WorkerResponse::Open(open(options).await),
            WorkerRequest::Run(options) => WorkerResponse::Run(run(options).await),
            WorkerRequest::LoadDb(options) => WorkerResponse::LoadDb(load_db(options).await),
        };
        if let Err(err) = scope.post_message(&serde_wasm_bindgen::to_value(&resp).unwrap()) {
            log::error!("Failed to send response: {err:?}");
        }
    }
}

pub fn entry() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<JsValue>();

    let scope: DedicatedWorkerGlobalScope = JsValue::from(js_sys::global()).into();
    spawn_local(execute_task(scope.clone(), rx));

    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
        tx.send(ev.data()).unwrap();
    });

    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    scope
        .post_message(&serde_wasm_bindgen::to_value(&WorkerResponse::Ready).unwrap())
        .expect("Failed to send ready");
    on_message.forget();
}
