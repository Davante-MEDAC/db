use leptos::prelude::*;
use reactive_stores::Store;

use crate::{
    LoadDbOptions, RunOptions, WorkerRequest,
    app::{AppState, AppStateStoreFields, embedded::EMBEDDED_DBS},
    send_request,
};

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <aside class="sidebar">
            <div class="sidebar-section-header">"Databases"</div>
            <div class="sidebar-table-list">
                {EMBEDDED_DBS
                    .iter()
                    .map(|db| {
                        let name = db.name;
                        let bytes = db.data;
                        let is_active = move || {
                            state.db_filename().read().as_deref() == Some(name)
                        };
                        let on_click = move |_| {
                            let data = js_sys::Uint8Array::from(bytes);
                            state.db_filename().set(Some(name.to_string()));
                            state.tables().set(vec![]);
                            state.selected_table().set(None);
                            state.table_columns().set(vec![]);
                            state.table_rows().set(vec![]);
                            send_request(state, WorkerRequest::LoadDb(LoadDbOptions { data }));
                        };
                        view! {
                            <button
                                class=move || {
                                    if is_active() {
                                        "sidebar-table-item selected"
                                    } else {
                                        "sidebar-table-item"
                                    }
                                }
                                on:click=on_click
                            >
                                <span class="table-icon">"🗄"</span>
                                {name}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Show when=move || !state.tables().read().is_empty() fallback=|| ()>
                <div class="sidebar-section-header sidebar-section-header--top-border">"Tables"</div>
                <div class="sidebar-table-list">
                    <For
                        each=move || state.tables().read().clone()
                        key=|t| t.clone()
                        children=move |table_name| {
                            let tn = table_name.clone();
                            let is_selected =
                                move || state.selected_table().read().as_deref() == Some(&tn);
                            let on_click = {
                                let tn2 = table_name.clone();
                                move |_| {
                                    send_request(
                                        state,
                                        WorkerRequest::Run(RunOptions {
                                            sql: format!(
                                                "SELECT * FROM \"{}\" LIMIT 500",
                                                tn2
                                            ),
                                            tag: format!("data:{}", tn2),
                                        }),
                                    );
                                }
                            };
                            view! {
                                <button
                                    class=move || {
                                        if is_selected() {
                                            "sidebar-table-item selected"
                                        } else {
                                            "sidebar-table-item"
                                        }
                                    }
                                    on:click=on_click
                                >
                                    <span class="table-icon">"▤"</span>
                                    {table_name}
                                </button>
                            }
                        }
                    />
                </div>
            </Show>
        </aside>
    }
}
