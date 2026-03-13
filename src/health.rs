use axum::{routing::get, Json, Router};
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn start_health_server() {
    let app = Router::new().route("/health", get(health_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000")
        .await
        .expect("Failed to bind health server to port 5000");

    info!("Health server listening on http://0.0.0.0:5000/health");
    axum::serve(listener, app)
        .await
        .expect("Health server crashed");
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
