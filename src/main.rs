use anyhow;
use axum::{
    routing::get,
    Router
};
use std::sync::Arc;

use fotos_backend::{handlers, AppState};

static APPNAME : &str = "foto_backend";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // FIXME: add cli parameters for verbose
    // FIXME: add linter to the project

    let cfg_path = confy::get_configuration_file_path(APPNAME, None)
        .unwrap();
    println!("Load config {}", cfg_path.to_str().unwrap_or(""));

    let app_conf : AppState = confy::load(APPNAME, None)?;
    let shared_state = Arc::new(app_conf);

    let app = Router::new()
        .route("/*subpath", get(handlers::download))
        .route("/", get(handlers::download))
        .with_state(shared_state);

    // FIXME: make host and port configurable
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
