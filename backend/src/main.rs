use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use dotenvy::dotenv;
use serde_json::json;
use std::env;
mod modules;

#[derive(Debug)]
enum ApiError {
    NotFound, // 404 Not Found
}

// Implementasi IntoResponse untuk ApiError agar bisa dikonversi menjadi response HTTP
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// Handler untuk root endpoint
async fn root() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Server is running",
    }))
}

// Handler untuk health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Health check passed",
    }))
}

// Handler untuk fallback route (not found)
async fn not_found() -> ApiError {
    ApiError::NotFound
}

// Fungsi untuk membaca port dari environment variable (default 3000)
fn read_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(3000)
}

// Fungsi untuk membaca host dari environment variable (default 0.0.0.0)
fn read_host() -> String {
    env::var("HOST")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "0.0.0.0".to_string())
}

fn create_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .fallback(not_found)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = create_app();
    let port = read_port();
    let host = read_host();
    let bind_addr = format!("{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

    println!("Server running on http://{}", bind_addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
