use anyhow;
use axum::{
    routing::get,
    Router
};
use std::sync::Arc;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use fotos_backend::{handlers, AppState};

static APPNAME : &str = "foto_backend";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: add cli parameters for verbose
    // TODO: add linter to the project

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let cfg_path = confy::get_configuration_file_path(APPNAME, None)
        .unwrap();
    tracing::info!("Load config {}", cfg_path.to_str().unwrap_or(""));

    let app_conf : AppState = confy::load(APPNAME, None)?;
    let shared_state = Arc::new(app_conf);

    let app = Router::new()
        .route("/*subpath", get(handlers::download))
        .route("/", get(handlers::download))
        .with_state(shared_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
        );

    // run it with hyper on localhost:3000
    // TODO: make host and port configurable
    let addr = "0.0.0.0:3000".parse()?;
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
