use leptos::prelude::*;
use reactive_stores::Store;

use crate::app::{AppState, AppStateStoreFields};

#[component]
pub fn TableView() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <main class="main-content">
            <Show
                when=move || state.selected_table().read().is_some()
                fallback=move || {
                    view! {
                        <div class="no-table-selected">
                            <p>"Select a table from the sidebar to view its data."</p>
                        </div>
                    }
                }
            >
                <TableHeader />
                <div class="table-tabs">
                    <button class="tab active">"Sample Data"</button>
                </div>
                <DataGrid />
            </Show>
        </main>
    }
}

#[component]
fn TableHeader() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="table-header">
            <div class="table-breadcrumb">
                <span class="breadcrumb-item">
                    {move || state.db_filename().read().clone().unwrap_or_else(|| "database".into())}
                </span>
                <span class="breadcrumb-sep">" / "</span>
                <span class="breadcrumb-item active">
                    {move || state.selected_table().read().clone().unwrap_or_default()}
                </span>
            </div>
            <h1 class="table-name">
                {move || state.selected_table().read().clone().unwrap_or_default()}
            </h1>
        </div>
    }
}

#[component]
fn DataGrid() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="data-grid-container">
            <Show
                when=move || !state.table_columns().read().is_empty()
                fallback=|| view! { <div class="empty-table">"No data"</div> }
            >
                <div class="data-grid-scroll">
                    <table class="data-grid">
                        <thead>
                            <tr>
                                <th class="row-num-col">"#"</th>
                                <For
                                    each=move || {
                                        let t = state.selected_table().read().clone().unwrap_or_default();
                                        state.table_columns().read().clone().into_iter()
                                            .map(move |c| format!("{t}:{c}"))
                                            .collect::<Vec<_>>()
                                    }
                                    key=|k| k.clone()
                                    children=|k| {
                                        let col = k.splitn(2, ':').nth(1).unwrap_or("").to_string();
                                        view! { <th>{col}</th> }
                                    }
                                />
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || {
                                    let t = state.selected_table().read().clone().unwrap_or_default();
                                    state.table_rows().read().clone().into_iter().enumerate()
                                        .map(move |(i, row)| (format!("{t}:{i}"), i, row))
                                        .collect::<Vec<_>>()
                                }
                                key=|(k, _, _)| k.clone()
                                children=|(_, i, row)| {
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
                        let rows = state.table_rows().read().len();
                        let cols = state.table_columns().read().len();
                        format!("{rows} rows × {cols} columns")
                    }}
                </div>
            </Show>
        </div>
    }
}
