use axum::{
    routing::get,
    Router
};
use sqlx::sqlite::SqlitePoolOptions;
use std::{sync::Arc, str::FromStr};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use fotos_backend::{handlers, AppConf, AppState};

static APPNAME : &str = "foto_backend";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let cfg_path = confy::get_configuration_file_path(APPNAME, None)
        .unwrap();
    let app_conf : AppConf = confy::load(APPNAME, None)?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://test.db")
        .await?;

    let app_state = AppState {
        conf: app_conf,
        pool
    };

    // Set up tracing and logging
    let max_level: Level = Level::from_str(app_state.conf.max_level.as_str())
        .unwrap_or(Level::INFO);
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(max_level)
        .compact()
        .init();

    tracing::debug!("Loaded config {}", cfg_path.to_str().unwrap_or(""));

    // Setup routes
    let addr = app_state.conf.connection.parse()?;
    let shared_state = Arc::new(app_state);
    let app = Router::new()
        .route("/data/*subpath", get(handlers::download))
        .route("/data", get(handlers::download))
        .with_state(shared_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
        );

    // Start server
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
