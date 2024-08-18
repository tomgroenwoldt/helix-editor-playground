use axum::{
    http::{header::CONTENT_TYPE, HeaderValue},
    routing::get,
    Router,
};
use helix::{get_versions, tutor};
use tower_http::cors::CorsLayer;

mod error;
mod helix;
mod terminal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // Allow users to connect from my personal GitHub pages
    let cors = CorsLayer::new()
        .allow_headers([CONTENT_TYPE])
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap());

    let app = Router::new()
        .route("/helix/:version", get(tutor))
        .route("/versions", get(get_versions))
        .layer(cors);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
