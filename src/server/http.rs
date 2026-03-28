// This file contains the HTTP server bootstrap, routing, and request handlers.
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;

use crate::database;

use super::dto::{
    ErrorResponse, HealthResponse, SyncAckRequest, SyncAckResponse, SyncPullQuery, SyncPullResponse,
    SyncPushRequest, SyncPushResponse,
};
use super::service::SyncService;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8787";

pub async fn spawn() -> anyhow::Result<()> {
    HttpServer::new(Arc::new(SyncService::new(database::database_path())))
        .spawn()
        .await
}

#[derive(Clone)]
struct AppState {
    service: Arc<SyncService>,
}

pub(super) struct HttpServer {
    state: AppState,
}

impl HttpServer {
    fn new(service: Arc<SyncService>) -> Self {
        Self {
            state: AppState { service },
        }
    }

    async fn spawn(self) -> anyhow::Result<()> {
        let app = self.router();
        let listener = tokio::net::TcpListener::bind(Self::bind_addr()?).await?;

        tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("sync server stopped: {error}");
            }
        });

        Ok(())
    }

    fn bind_addr() -> anyhow::Result<SocketAddr> {
        std::env::var("TURN_CHECKER_BIND_ADDR")
            .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string())
            .parse()
            .map_err(|error| anyhow::anyhow!("invalid bind address: {error}"))
    }

    fn router(&self) -> Router {
        Router::new()
            .route("/health", get(Self::health))
            .route("/sync/pull", get(Self::sync_pull))
            .route("/sync/push", post(Self::sync_push))
            .route("/sync/ack", post(Self::sync_ack))
            .with_state(self.state.clone())
    }

    async fn health() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "ok",
            server_time: Utc::now(),
        })
    }

    async fn sync_pull(
        State(state): State<AppState>,
        Query(query): Query<SyncPullQuery>,
    ) -> Result<Json<SyncPullResponse>, AppError> {
        Ok(Json(state.service.pull(query.limit)?))
    }

    async fn sync_push(
        State(state): State<AppState>,
        Json(request): Json<SyncPushRequest>,
    ) -> Result<Json<SyncPushResponse>, AppError> {
        Ok(Json(state.service.push(request)?))
    }

    async fn sync_ack(
        State(state): State<AppState>,
        Json(request): Json<SyncAckRequest>,
    ) -> Result<Json<SyncAckResponse>, AppError> {
        Ok(Json(state.service.ack(request)?))
    }
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
