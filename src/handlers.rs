use crate::AppState;

use axum::extract::State;
use std::sync::Arc;

// FIXME: get the folder from the request path
pub async fn list_folder(State(cfg): State<Arc<AppState>>) -> String {
    // FIXME: return the children

    cfg.root.clone()
}
