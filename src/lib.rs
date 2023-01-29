use serde::{Serialize, Deserialize};

pub mod handlers;

pub static APPNAME : &str = "foto_backend";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConf {
    pub root: String
}

impl Default for AppConf {
    fn default() -> Self {
        Self { root: "./data".to_string() }
    }
}