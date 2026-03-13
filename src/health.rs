use crate::metrics::Metrics;
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn start_health_server(metrics: Arc<Metrics>) {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(metrics);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000")
        .await
        .expect("Failed to bind health server to port 5000");

    info!("Health server listening on http://0.0.0.0:5000");
    info!("  GET /health  — liveness check");
    info!("  GET /metrics — Prometheus metrics");

    axum::serve(listener, app)
        .await
        .expect("Health server crashed");
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn metrics_handler(State(metrics): State<Arc<Metrics>>) -> String {
    metrics.render()
}
