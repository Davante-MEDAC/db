use leptos::prelude::*;
use reactive_stores::Store;

use crate::app::{AppState, AppStateStoreFields};

#[component]
pub fn TableView() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="flex flex-col flex-1 overflow-hidden">
            <Show
                when=move || state.selected_table().read().is_some()
                fallback=move || view! {
                    <div class="flex flex-1 items-center justify-center text-slate-500 text-sm">
                        "Select a table from the sidebar."
                    </div>
                }
            >
                <TableHeader />
                // Sample Data tab bar
                <div class="flex items-center bg-slate-800 border-b border-slate-700 px-6 shrink-0">
                    <button class="px-4 py-2.5 text-sm font-medium text-blue-400 border-b-2 border-blue-400 -mb-px">
                        "Sample Data"
                    </button>
                </div>
                <DataGrid />
            </Show>
        </div>
    }
}

#[component]
fn TableHeader() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="bg-slate-800 border-b border-slate-700 px-6 pt-4 pb-0 shrink-0">
            // Breadcrumb
            <div class="flex items-center gap-1 text-xs text-slate-500 mb-2">
                <span>
                    {move || state.db_filename().read().clone().unwrap_or_else(|| "database".into())}
                </span>
                <span class="text-slate-600">"/"</span>
                <span class="text-slate-300">
                    {move || state.selected_table().read().clone().unwrap_or_default()}
                </span>
            </div>
            // Table name
            <h1 class="text-xl font-semibold text-slate-100 pb-4 tracking-tight">
                {move || state.selected_table().read().clone().unwrap_or_default()}
            </h1>
        </div>
    }
}

#[component]
fn DataGrid() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="flex flex-col flex-1 overflow-hidden p-4 gap-2">
            <Show
                when=move || !state.table_columns().read().is_empty()
                fallback=|| view! { <div class="text-slate-500 text-sm p-4">"No data"</div> }
            >
                <div class="flex-1 overflow-auto rounded border border-slate-700 bg-slate-800">
                    <table class="w-full text-sm border-collapse">
                        <thead class="sticky top-0 z-10">
                            <tr>
                                <th class="bg-slate-700 text-slate-400 text-xs font-semibold tracking-wide text-right px-3 py-2 border-b border-slate-600 border-r border-slate-600 w-10 select-none">
                                    "#"
                                </th>
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
                                        view! {
                                            <th class="bg-slate-700 text-slate-400 text-xs font-semibold tracking-wide text-left px-3 py-2 border-b border-slate-600 border-r border-slate-600 last:border-r-0 whitespace-nowrap">
                                                {col}
                                            </th>
                                        }
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
                                        <tr class="hover:bg-slate-700/40 even:bg-slate-800/60">
                                            <td class="text-slate-600 text-xs text-right px-3 py-1.5 border-b border-slate-700/50 border-r border-slate-700/50 select-none font-mono">
                                                {i + 1}
                                            </td>
                                            {row.into_iter()
                                                .map(|cell| view! {
                                                    <td class="text-slate-200 px-3 py-1.5 border-b border-slate-700/50 border-r border-slate-700/50 last:border-r-0 cell-truncate">
                                                        {cell}
                                                    </td>
                                                })
                                                .collect_view()}
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
                <div class="text-xs text-slate-500 px-1">
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
