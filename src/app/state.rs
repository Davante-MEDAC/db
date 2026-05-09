use aceditor::Editor;
use reactive_stores::Store;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MainView {
    #[default]
    Browse,
    Query,
}

#[derive(Store, Serialize, Deserialize, Default)]
pub struct AppState {
    pub db_loaded: bool,
    pub db_filename: Option<String>,
    pub tables: Vec<String>,
    pub selected_table: Option<String>,
    pub table_columns: Vec<String>,
    pub table_rows: Vec<Vec<String>>,
    pub query_columns: Vec<String>,
    pub query_rows: Vec<Vec<String>>,
    pub query_error: Option<String>,
    pub main_view: MainView,
    pub error: Option<String>,
    #[serde(skip)]
    pub editor: Option<Editor>,
}
