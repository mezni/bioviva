use axum::{
    Router,
    routing::get,
    Json,
};
use serde::Serialize;

fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}

#[tokio::main]
async fn main() {
    let app = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}