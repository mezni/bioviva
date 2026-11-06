use axum::{
    Router,
    routing::{get, post},
    Extension,
    middleware,
    Json,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt::Layer};

use crate::infrastructure::jwt::JwtService;

#[derive(serde::Deserialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(serde::Deserialize)]
struct RegisterPayload {
    email: String,
    password: String,
}

#[derive(serde::Serialize)]
struct RegisterResponse {
    user_id: uuid::Uuid,
}

fn create_router(jwt: JwtService) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(handler_auth::register))
        .route("/auth/login", post(handler_auth::login))
        .route("/products", get(handler_product::list))
        .with_state(AppState { jwt })
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(auth_middleware))
        )
        .layer(EnvFilter::try_from_default_env().unwrap_or_else(|_| "bioivara=info".into()))
}

#[axum::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "bioivara=info".into()))
        .init();

    let app = create_router(crate::infrastructure::jwt::JwtService::new(
        b"bioviva-jwt-secret-must-be-at-least-32-bytes-secret-key",
    ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

struct AppState {
    jwt: JwtService,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn auth_middleware(
    headers: axum::http::HeaderMap,
    next: axum::middleware::Next,
) -> Result<impl IntoResponse, ApiError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .ok_or(ApiError::Unauthorized("Missing bearer token".into()))?;

    let token = &auth_header[7..];

    // In a real impl, validate the token here
    let _user_id = uuid::Uuid::new_v4(); // placeholder

    let response = next.run(request).await;
    Ok(response)
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            ApiError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            ApiError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            ApiError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, body).into_response()
    }
}

// ===== Handler modules =====

mod handler_auth {
    use super::*;

    pub async fn register(
        Extension(app_state): Extension<AppState>,
        Json(payload): Json<RegisterPayload>,
    ) -> Result<Json<RegisterResponse>, ApiError> {
        // Placeholder: would call application use case
        let user_id = uuid::Uuid::new_v4();
        Ok(Json(RegisterResponse { user_id }))
    }

    pub async fn login(
        Extension(app_state): Extension<AppState>,
        Json(payload): Json<LoginPayload>,
    ) -> Result<Json<LoginResponse>, ApiError> {
        // Placeholder: would verify credentials and issue JWT
        let token = app_state.jwt.generate_access_token(uuid::Uuid::new_v4(), "consumer");
        Ok(Json(LoginResponse { token }))
    }
}

mod handler_product {
    use super::*;

    pub async fn list() -> Json<serde_json::Value> {
        Json(serde_json::json!({ "products": [] }))
    }
}

// ===== Module exports for router =====

mod auth_routes {
    use super::*;

    pub fn routes() -> axum::Router {
        Router::new()
            .route("/register", post(handler_auth::register))
            .route("/login", post(handler_auth::login))
    }
}

mod product_routes {
    use super::*;

    pub fn routes() -> axum::Router {
        Router::new().route("/products", get(handler_product::list))
    }
}