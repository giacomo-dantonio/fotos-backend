use serde::{Serialize, Deserialize};

pub mod api;
pub mod handlers;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub root: String,
    pub connection: String,
    pub max_level: String
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            root: "./data".to_string(),
            connection: "0.0.0.0:3000".to_string(),
            max_level: "INFO".to_string()
        }
    }
}
