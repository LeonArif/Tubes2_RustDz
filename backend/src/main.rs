use axum::{
    Json, Router,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

use crate::modules::domnode::DomNode;
use crate::modules::selector::{SelectorQuery, find_matching_nodes};
use crate::modules::tree::Tree;
mod modules;

#[derive(Debug)]
enum ApiError {
    NotFound, // 404 Not Found
    BadRequest(String),
}

// Implementasi IntoResponse untuk ApiError agar bisa dikonversi menjadi response HTTP
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
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

#[derive(Debug, Deserialize)]
struct TraversalRequest {
    html_content: Option<String>,
    source_url: Option<String>,
    css_selector: String,
    method: String,
}

#[derive(Debug, Serialize)]
struct TraversalMatchedNode {
    id: String,
    tag: String,
    class: String,
}

#[derive(Debug, Serialize)]
struct TraversalResponse {
    execution_time_us: u64,
    matched_nodes: Vec<TraversalMatchedNode>,
    traversal_path: Vec<String>,
    tree_data: serde_json::Value,
}

// fungsi parsing HTML menjadi Tree menggunakan pendekatan struktur data stack
async fn traverse(Json(payload): Json<TraversalRequest>) -> Result<impl IntoResponse, ApiError> {
    // HTML bisa dikirim langsung atau diambil dari URL
    let html_content = resolve_html_source(&payload).await?;

    if payload.css_selector.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "css_selector cannot be empty".to_string(),
        ));
    }

    // Parsing HTML ke dalam struktur Tree menggunakan parser custom yang toleran terhadap markup yang tidak sempurna. Pendekatan ini lebih stabil untuk berbagai input HTML, termasuk fragmen dan markup yang tidak valid, dibandingkan dengan parser berbasis regex sederhana.
    let tree = modules::parser::parse_html_to_tree(&html_content).map_err(ApiError::BadRequest)?;

    // Parsing CSS selector menggunakan modul selector untuk menghasilkan struktur query yang dapat dieksekusi oleh algoritma traversal
    // Validasi dilakukan untuk memastikan selector yang diberikan sesuai dengan subset CSS yang didukung,
    // dan error handling memberikan feedback yang jelas jika terjadi kesalahan dalam parsing selector.
    let selector_query =
        SelectorQuery::parse(payload.css_selector.trim()).map_err(ApiError::BadRequest)?;

    let method = payload.method.trim().to_ascii_uppercase();
    // algoritma traversal dipilih berdasarkan input method,
    // eksekusi traversal menghasilkan urutan node yang dikunjungi serta metrik waktu eksekusi,
    // yang kemudian digunakan untuk membangun response yang dikirim kembali ke frontend
    let (traversal_order, elapsed_ms) = match method.as_str() {
        "BFS" => {
            let query = modules::bfs::SearchQuery::Selector(selector_query.clone());
            let result = modules::bfs::bfs(&tree, &query);
            (result.traversal_order, result.metrics.elapsed_ms)
        }
        "DFS" => {
            let query = modules::dfs::SearchQuery::Selector(selector_query.clone());
            let result = modules::dfs::dfs(&tree, &query);
            (result.traversal_order, result.metrics.elapsed_ms)
        }
        _ => {
            return Err(ApiError::BadRequest(
                "method must be either BFS or DFS".to_string(),
            ));
        }
    };

    // mengumpulkan semua node yang cocok dengan selector dari hasil traversal,
    let matched_ids = find_matching_nodes(&tree, &selector_query);
    let matched_nodes = matched_ids
        .into_iter()
        .filter_map(|id| {
            let index = tree.find_index_from_id(id)?;
            let node = tree.nodes.get(index)?;
            Some(to_matched_node(id, node))
        })
        .collect::<Vec<_>>();

    // execution time dari proses traversal
    let tree_data = build_tree_data(&tree);
    let execution_time_us_u128 = elapsed_ms.saturating_mul(1000);
    let execution_time_us = execution_time_us_u128.min(u128::from(u64::MAX)) as u64;

    let response = TraversalResponse {
        execution_time_us,
        matched_nodes,
        traversal_path: traversal_order
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        tree_data,
    };

    Ok(Json(response))
}

// fungsi untuk mengambil konten HTML dari URL atau langsung dari input, 
// dengan validasi dan error handling untuk memastikan konten yang diproses valid dan dapat digunakan untuk parsing selanjutnya
// Pendekatan ini memberikan fleksibilitas bagi user untuk memilih sumber HTML baik itu melalui URL maupun input langsung teks HTML-nya
async fn resolve_html_source(payload: &TraversalRequest) -> Result<String, ApiError> {
    if let Some(source_url) = payload.source_url.as_deref() {
        let source_url = source_url.trim();
        if source_url.is_empty() {
            return Err(ApiError::BadRequest(
                "source_url cannot be empty".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| {
                ApiError::BadRequest("Failed to initialize URL fetch client".to_string())
            })?;

        let response = client
            .get(source_url)
            .send()
            .await
            .map_err(|_| ApiError::BadRequest("Gagal mengambil konten dari URL".to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::BadRequest(format!(
                "URL merespons dengan status {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|_| ApiError::BadRequest("Gagal membaca konten dari URL".to_string()))?;

        if body.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "Konten HTML dari URL kosong".to_string(),
            ));
        }

        return Ok(body);
    }

    if let Some(html_content) = payload.html_content.as_deref() {
        if !html_content.trim().is_empty() {
            return Ok(html_content.to_string());
        }
    }

    Err(ApiError::BadRequest(
        "source_url or html_content is required".to_string(),
    ))
}

// fungsi pencocokan node
fn to_matched_node(id: usize, node: &DomNode) -> TraversalMatchedNode {
    let class_value = node
        .attrs()
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("class"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();

    TraversalMatchedNode {
        id: id.to_string(),
        tag: node.tag().to_string(),
        class: class_value,
    }
}

// fungsi membangun data tree
fn build_tree_data(tree: &Tree) -> serde_json::Value {
    // serialize tree ke dalam format JSON untuk diterima frontend,
    // termasuk informasi tentang setiap node seperti id, tag, parent, children, depth, dan atribut
    // Struktur ini memungkinkan frontend untuk merekonstruksi kembali pohon DOM dan menampilkan informasi yang relevan
    let nodes = tree
        .nodes
        .iter()
        .map(|node| {
            let attrs = node
                .attrs()
                .iter()
                .map(|(key, value)| json!({ "key": key, "value": value }))
                .collect::<Vec<_>>();

            json!({
                "id": node.id().to_string(),
                "tag": node.tag(),
                "parent": node.parent().map(|id| id.to_string()),
                "children": node.children().iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "depth": node.depth(),
                "attrs": attrs,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "root_id": tree.root.to_string(),
        "nodes": nodes,
    })
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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/traverse", post(traverse))
        .fallback(not_found)
        .layer(cors)
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
