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
        <div class="app-shell">
            <header::Header />
            <div class="app-body">
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
        <div class="main-area">
            <div class="main-mode-tabs">
                <button
                    class=move || if *state.main_view().read() == MainView::Browse { "mode-tab active" } else { "mode-tab" }
                    on:click=move |_| state.main_view().set(MainView::Browse)
                >
                    "Browse"
                </button>
                <button
                    class=move || if *state.main_view().read() == MainView::Query { "mode-tab active" } else { "mode-tab" }
                    on:click=move |_| state.main_view().set(MainView::Query)
                >
                    "Query"
                </button>
            </div>
            <div class="main-view-content">
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
        <div class="landing">
            <div class="landing-card">
                <div class="landing-icon">"🗄️"</div>
                <h2>"Load a SQLite Database"</h2>
                <p>"Use the \"Open Database\" button in the top bar to load a .db or .sqlite file."</p>
            </div>
        </div>
    }
}
