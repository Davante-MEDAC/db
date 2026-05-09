use aceditor::{BindKey, EditorOptionsBuilder};
use leptos::prelude::*;
use reactive_stores::Store;
use wasm_bindgen::{JsCast, prelude::Closure};

use crate::app::{AppState, AppStateStoreFields};

pub fn run_query(state: Store<AppState>) -> Box<dyn Fn() + Send + 'static> {
    Box::new(move || {
        let guard = state.editor().read_untracked();
        let Some(editor) = guard.as_ref() else { return };
        let sql = editor.get_value();
        drop(guard);

        if sql.trim().is_empty() {
            return;
        }

        state.query_error().set(None);
        state.query_columns().set(vec![]);
        state.query_rows().set(vec![]);

        crate::send_request(
            state,
            crate::WorkerRequest::Run(crate::RunOptions {
                sql,
                tag: "query".into(),
            }),
        );
    })
}

#[component]
pub fn SqlEditor() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();
    let editor_ref = NodeRef::new();

    editor_ref.on_load(move |_| {
        let opt = EditorOptionsBuilder::default()
            .mode("ace/mode/sql")
            .theme("ace/theme/github")
            .value("SELECT * FROM sqlite_master;")
            .build();

        match aceditor::Editor::open("ace_editor", Some(&opt)) {
            Ok(editor) => {
                let exec = Closure::<dyn Fn() + 'static>::new(run_query(state));
                editor.add_command(aceditor::Command {
                    name: "runQuery".into(),
                    bind_key: BindKey {
                        win: "Ctrl-Enter".into(),
                        mac: "Ctrl-Enter|Command-Enter".into(),
                    },
                    exec: exec.as_ref().unchecked_ref::<js_sys::Function>().clone(),
                    read_only: false,
                });
                exec.forget();
                state.editor().set(Some(editor));
            }
            Err(err) => {
                log::error!("Failed to open ace editor: {err:?}");
            }
        }
    });

    view! {
        <div node_ref=editor_ref id="ace_editor" class="ace-editor-container"></div>
    }
}
