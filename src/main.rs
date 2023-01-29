use anyhow;
use axum::{
    routing::get,
    Router
};

use fotos_backend::{APPNAME, handlers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // FIXME: add cli parameters for verbose

    let cfg_path = confy::get_configuration_file_path(APPNAME, None)
        .unwrap();
    println!("Load config {}", cfg_path.to_str().unwrap_or(""));

    let app = Router::new()
    .route("/", get(handlers::list_folder));

    // FIXME: make host and port configurable
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
