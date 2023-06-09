use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

pub mod api;
pub mod handlers;
pub mod infrastructure;

#[cfg(test)]
pub mod test_utils;

/// The configuration of the application.
/// Will be serialized to and deserialized from toml using the `confy` crate.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConf {
    /// The root folder of the content that will be served through the server.
    pub root: String,

    /// The address the server will listen on (e.g. `0.0.0.0:3000`).
    pub connection: String,

    /// The maximum level used for logging. Can be one of the following
    /// (see the [https://docs.rs/tracing-core/latest/tracing_core/struct.Level.html#implementations](documentation)
    /// for more information):
    ///
    /// - `OFF`
    /// - `ERROR`
    /// - `WARN`
    /// - `INFO`
    /// - `DEBUG`
    /// - `TRACE`
    pub max_level: String
}

pub struct AppState {
    pub conf: AppConf,
    pub pool: SqlitePool
}

impl Default for AppConf {
    fn default() -> Self {
        Self {
            root: "./data".to_string(),
            connection: "0.0.0.0:3000".to_string(),
            max_level: "INFO".to_string()
        }
    }
}
