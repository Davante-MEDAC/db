mod editor;
pub mod embedded;
mod header;
mod query;
mod sidebar;
mod state;
mod table_view;

pub use state::*;

use leptos::prelude::*;
use reactive_stores::Store;

#[component]
pub fn App() -> impl IntoView {
    provide_context(Store::new(AppState::default()));
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="flex flex-col h-screen overflow-hidden">
            <header::Header />
            <div class="flex flex-1 overflow-hidden">
                <sidebar::Sidebar />
                <Show
                    when=move || *state.db_loaded().read()
                    fallback=|| view! { <LandingPage /> }
                >
                    <MainArea />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn MainArea() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();

    view! {
        <div class="flex flex-col flex-1 overflow-hidden bg-slate-900">
            // Mode tabs
            <div class="flex items-center bg-slate-800 border-b border-slate-700 px-4 shrink-0">
                <button
                    class=move || {
                        if *state.main_view().read() == MainView::Browse {
                            "px-4 py-2.5 text-sm font-medium text-blue-400 border-b-2 border-blue-400 -mb-px"
                        } else {
                            "px-4 py-2.5 text-sm font-medium text-slate-400 border-b-2 border-transparent -mb-px hover:text-slate-200"
                        }
                    }
                    on:click=move |_| state.main_view().set(MainView::Browse)
                >
                    "Browse"
                </button>
                <button
                    class=move || {
                        if *state.main_view().read() == MainView::Query {
                            "px-4 py-2.5 text-sm font-medium text-blue-400 border-b-2 border-blue-400 -mb-px"
                        } else {
                            "px-4 py-2.5 text-sm font-medium text-slate-400 border-b-2 border-transparent -mb-px hover:text-slate-200"
                        }
                    }
                    on:click=move |_| state.main_view().set(MainView::Query)
                >
                    "Query"
                </button>
            </div>
            // View content
            <div class="flex flex-col flex-1 overflow-hidden">
                <Show
                    when=move || *state.main_view().read() == MainView::Browse
                    fallback=|| view! { <query::QueryPanel /> }
                >
                    <table_view::TableView />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn LandingPage() -> impl IntoView {
    view! {
        <div class="flex flex-1 items-center justify-center bg-slate-900">
            <div class="text-center p-12 bg-slate-800 border border-slate-700 rounded-xl shadow-xl max-w-sm">
                <div class="text-5xl mb-4">"🗄️"</div>
                <h2 class="text-lg font-semibold text-slate-100 mb-2">"Load a SQLite Database"</h2>
                <p class="text-sm text-slate-400 leading-relaxed">
                    "Click a database in the sidebar or use \"Open Database\" to load a file."
                </p>
            </div>
        </div>
    }
}
