// This file contains the HTTP server bootstrap, routing, and request handlers.
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;

use crate::database;

use super::dto::{
    ErrorResponse, HealthResponse, SyncAckRequest, SyncAckResponse, SyncConnectRequest,
    SyncConnectResponse, SyncPullQuery, SyncPullResponse, SyncPushRequest, SyncPushResponse,
};
use super::pairing::PairingState;
use super::service::SyncService;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8787";

#[derive(Debug, Clone)]
pub struct ServerConnectionInfo {
    pub base_url: String,
    pub qr_payload: String,
}

pub async fn spawn(pairing_state: PairingState) -> anyhow::Result<ServerConnectionInfo> {
    HttpServer::new(
        Arc::new(SyncService::new(database::database_path())),
        pairing_state,
    )
        .spawn()
        .await
}

#[derive(Clone)]
struct AppState {
    service: Arc<SyncService>,
    pairing_state: PairingState,
}

pub(super) struct HttpServer {
    state: AppState,
}

impl HttpServer {
    fn new(service: Arc<SyncService>, pairing_state: PairingState) -> Self {
        Self {
            state: AppState {
                service,
                pairing_state,
            },
        }
    }

    async fn spawn(self) -> anyhow::Result<ServerConnectionInfo> {
        let app = self.router();
        let bind_addr = Self::bind_addr()?;
        let advertised_addr = Self::advertised_addr(bind_addr)?;
        let listener = tokio::net::TcpListener::bind(bind_addr).await?;

        tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("sync server stopped: {error}");
            }
        });

        let base_url = format!("http://{advertised_addr}");
        let qr_payload = serde_json::json!({
            "baseUrl": base_url,
            "version": 1
        })
        .to_string();

        Ok(ServerConnectionInfo {
            base_url,
            qr_payload,
        })
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
            .route("/sync/connect", post(Self::sync_connect))
            .route("/sync/pull", get(Self::sync_pull))
            .route("/sync/push", post(Self::sync_push))
            .route("/sync/ack", post(Self::sync_ack))
            .with_state(self.state.clone())
    }

    fn advertised_addr(bind_addr: SocketAddr) -> anyhow::Result<SocketAddr> {
        if bind_addr.ip().is_unspecified() {
            let advertised_ip = discover_local_ip()?;
            Ok(SocketAddr::new(advertised_ip, bind_addr.port()))
        } else {
            Ok(bind_addr)
        }
    }

    async fn health() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "ok",
            server_time: Utc::now(),
        })
    }

    async fn sync_connect(
        State(state): State<AppState>,
        Json(_request): Json<SyncConnectRequest>,
    ) -> Json<SyncConnectResponse> {
        state.pairing_state.mark_paired();
        Json(SyncConnectResponse {
            ok: true,
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

fn discover_local_ip() -> anyhow::Result<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip())
}
