use leptos::prelude::*;
use reactive_stores::Store;

use crate::app::{AppState, AppStateStoreFields, editor::{SqlEditor, run_query}};

#[component]
pub fn QueryPanel() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="query-panel">
            <div class="query-editor-section">
                <div class="query-toolbar">
                    <span class="query-toolbar-label">"SQL Editor"</span>
                    <button
                        class="btn-run"
                        on:click=move |_| run_query(state)()
                    >
                        "▶ Run"
                        <span class="query-hint">"Ctrl+Enter"</span>
                    </button>
                </div>
                <SqlEditor />
            </div>
            <div class="query-results-section">
                <Show
                    when=move || state.query_error().read().is_some()
                    fallback=|| ()
                >
                    <div class="query-error">
                        {move || state.query_error().read().clone().unwrap_or_default()}
                    </div>
                </Show>
                <Show
                    when=move || !state.query_columns().read().is_empty()
                    fallback=move || {
                        view! {
                            <div class="query-empty">
                                <Show
                                    when=move || state.query_error().read().is_none()
                                    fallback=|| ()
                                >
                                    <span>"Run a query to see results."</span>
                                </Show>
                            </div>
                        }
                    }
                >
                    <QueryResultGrid />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn QueryResultGrid() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="data-grid-scroll">
            <table class="data-grid">
                <thead>
                    <tr>
                        <th class="row-num-col">"#"</th>
                        <For
                            each=move || state.query_columns().read().clone()
                            key=|c| c.clone()
                            children=|col| view! { <th>{col}</th> }
                        />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || {
                            state.query_rows().read().clone().into_iter().enumerate().collect::<Vec<_>>()
                        }
                        key=|(i, _)| *i
                        children=|(i, row)| {
                            view! {
                                <tr>
                                    <td class="row-num">{i + 1}</td>
                                    {row.into_iter().map(|cell| view! { <td>{cell}</td> }).collect_view()}
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
        <div class="data-grid-footer">
            {move || {
                let rows = state.query_rows().read().len();
                let cols = state.query_columns().read().len();
                format!("{rows} rows × {cols} columns")
            }}
        </div>
    }
}
