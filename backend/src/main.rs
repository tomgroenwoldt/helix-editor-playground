use axum::{
    http::{header::CONTENT_TYPE, HeaderValue},
    routing::get,
    Router,
};
use helix::{editor, get_versions};
use tower_http::cors::CorsLayer;

mod error;
mod helix;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // Allow users to connect from my personal GitHub pages
    let cors = CorsLayer::new().allow_headers([CONTENT_TYPE]).allow_origin(
        "https://tomgroenwoldt.github.io"
            .parse::<HeaderValue>()
            .unwrap(),
    );

    let app = Router::new()
        .route("/helix/:version", get(editor))
        .route("/versions", get(get_versions))
        .layer(cors);

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
