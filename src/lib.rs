use serde::{Serialize, Deserialize};

pub mod api;
pub mod handlers;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub root: String
}

impl Default for AppState {
    fn default() -> Self {
        Self { root: "./data".to_string() }
    }
}
