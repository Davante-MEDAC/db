use leptos::prelude::*;
use reactive_stores::Store;

use crate::app::{AppState, AppStateStoreFields, editor::{SqlEditor, run_query}};

#[component]
pub fn QueryPanel() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="flex flex-col flex-1 overflow-hidden">
            // Editor section
            <div class="flex flex-col shrink-0 border-b border-slate-700" style="height: 240px">
                <div class="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700 shrink-0">
                    <span class="text-xs font-semibold tracking-widest text-slate-500 uppercase">
                        "SQL Editor"
                    </span>
                    <button
                        class="flex items-center gap-2 bg-green-700 hover:bg-green-600 text-white text-xs font-medium px-3 py-1.5 rounded transition-colors"
                        on:click=move |_| run_query(state)()
                    >
                        "▶ Run"
                        <span class="text-green-300/70 font-mono">"Ctrl+Enter"</span>
                    </button>
                </div>
                <div class="flex-1 min-h-0">
                    <SqlEditor />
                </div>
            </div>

            // Results section
            <div class="flex flex-col flex-1 overflow-hidden p-4 gap-2">
                <Show when=move || state.query_error().read().is_some() fallback=|| ()>
                    <div class="bg-red-950 border border-red-800 text-red-300 text-xs font-mono px-4 py-3 rounded whitespace-pre-wrap shrink-0">
                        {move || state.query_error().read().clone().unwrap_or_default()}
                    </div>
                </Show>
                <Show
                    when=move || !state.query_columns().read().is_empty()
                    fallback=move || view! {
                        <div class="flex flex-1 items-center justify-center">
                            <Show
                                when=move || state.query_error().read().is_none()
                                fallback=|| ()
                            >
                                <span class="text-slate-500 text-sm">"Run a query to see results."</span>
                            </Show>
                        </div>
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
        <div class="flex-1 overflow-auto rounded border border-slate-700 bg-slate-800">
            <table class="w-full text-sm border-collapse">
                <thead class="sticky top-0 z-10">
                    <tr>
                        <th class="bg-slate-700 text-slate-400 text-xs font-semibold tracking-wide text-right px-3 py-2 border-b border-slate-600 border-r border-slate-600 w-10 select-none">
                            "#"
                        </th>
                        <For
                            each=move || state.query_columns().read().clone()
                            key=|c| c.clone()
                            children=|col| view! {
                                <th class="bg-slate-700 text-slate-400 text-xs font-semibold tracking-wide text-left px-3 py-2 border-b border-slate-600 border-r border-slate-600 last:border-r-0 whitespace-nowrap">
                                    {col}
                                </th>
                            }
                        />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || {
                            state.query_rows().read().clone().into_iter().enumerate().collect::<Vec<_>>()
                        }
                        key=|(i, _)| *i
                        children=|(i, row)| view! {
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
                    />
                </tbody>
            </table>
        </div>
        <div class="text-xs text-slate-500 px-1">
            {move || {
                let rows = state.query_rows().read().len();
                let cols = state.query_columns().read().len();
                format!("{rows} rows × {cols} columns")
            }}
        </div>
    }
}
