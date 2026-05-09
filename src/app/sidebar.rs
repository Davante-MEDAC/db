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
        <aside class="w-56 shrink-0 bg-slate-900 border-r border-slate-700 flex flex-col overflow-y-auto">

            // Databases section
            <div class="px-3 pt-4 pb-1 text-xs font-semibold tracking-widest text-slate-500 uppercase">
                "Databases"
            </div>
            <div class="px-2 pb-2">
                {EMBEDDED_DBS
                    .iter()
                    .map(|db| {
                        let name = db.name;
                        let bytes = db.data;
                        let is_active =
                            move || state.db_filename().read().as_deref() == Some(name);
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
                                        "w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-left \
                                         bg-blue-900/40 text-blue-300 font-medium"
                                    } else {
                                        "w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-left \
                                         text-slate-300 hover:bg-slate-800 hover:text-slate-100 transition-colors"
                                    }
                                }
                                on:click=on_click
                            >
                                <span class="text-slate-400 shrink-0">"🗄"</span>
                                <span class="truncate font-mono">{name}</span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            // Tables section — only when tables are loaded
            <Show when=move || !state.tables().read().is_empty() fallback=|| ()>
                <div class="mt-2 border-t border-slate-700/60 px-3 pt-3 pb-1 text-xs font-semibold tracking-widest text-slate-500 uppercase">
                    "Tables"
                </div>
                <div class="px-2 pb-4 flex flex-col gap-px">
                    <For
                        each=move || state.tables().read().clone()
                        key=|t| t.clone()
                        children=move |table_name| {
                            let tn = table_name.clone();
                            let is_selected = move || {
                                state.selected_table().read().as_deref() == Some(&tn)
                            };
                            let on_click = {
                                let tn2 = table_name.clone();
                                move |_| {
                                    send_request(
                                        state,
                                        WorkerRequest::Run(RunOptions {
                                            sql: format!("SELECT * FROM \"{}\" LIMIT 500", tn2),
                                            tag: format!("data:{}", tn2),
                                        }),
                                    );
                                }
                            };
                            view! {
                                <button
                                    class=move || {
                                        if is_selected() {
                                            "w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-left \
                                             bg-blue-900/40 text-blue-300 font-medium"
                                        } else {
                                            "w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-left \
                                             text-slate-400 hover:bg-slate-800 hover:text-slate-200 transition-colors"
                                        }
                                    }
                                    on:click=on_click
                                >
                                    <span class="text-slate-600 shrink-0 text-base leading-none">"▤"</span>
                                    <span class="truncate">{table_name}</span>
                                </button>
                            }
                        }
                    />
                </div>
            </Show>
        </aside>
    }
}
