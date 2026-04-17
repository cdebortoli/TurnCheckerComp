// This file contains the HTTP server bootstrap, routing, and request handlers.
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use tokio::sync::watch;

use crate::{database, i18n::I18n, models::CurrentSession};

use super::dto::{
    ErrorResponse, HealthResponse, SyncAckRequest, SyncAckResponse, SyncConnectRequest,
    SyncConnectResponse, SyncPullRequest, SyncPullResponse, SyncPushRequest, SyncPushResponse,
};
use super::notifications::PushNotificationClient;
use super::pairing::PairingState;
use super::service::SyncService;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8787";

#[derive(Debug, Clone)]
pub struct ServerConnectionInfo {
    pub base_url: String,
    pub qr_payload: String,
}

pub async fn spawn(
    pairing_state: PairingState,
    content_refresh_tx: watch::Sender<u64>,
    push_notification_client: PushNotificationClient,
) -> anyhow::Result<ServerConnectionInfo> {
    HttpServer::new(
        Arc::new(SyncService::new(database::database_path())),
        pairing_state,
        content_refresh_tx,
        push_notification_client,
    )
    .spawn()
    .await
}

#[derive(Clone)]
struct AppState {
    service: Arc<SyncService>,
    pairing_state: PairingState,
    content_refresh_tx: watch::Sender<u64>,
    push_notification_client: PushNotificationClient,
}

pub(super) struct HttpServer {
    state: AppState,
}

impl HttpServer {
    fn new(
        service: Arc<SyncService>,
        pairing_state: PairingState,
        content_refresh_tx: watch::Sender<u64>,
        push_notification_client: PushNotificationClient,
    ) -> Self {
        Self {
            state: AppState {
                service,
                pairing_state,
                content_refresh_tx,
                push_notification_client,
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
                let i18n = I18n::system();
                eprintln!(
                    "{}",
                    i18n.tr("server-stopped-log", &[("error", error.to_string().into())])
                );
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
            .map_err(|error: std::net::AddrParseError| {
                let i18n = I18n::system();
                anyhow::anyhow!(i18n.tr(
                    "server-invalid-bind-address",
                    &[("error", error.to_string().into())]
                ))
            })
    }

    fn router(&self) -> Router {
        Router::new()
            .route("/health", get(Self::health))
            .route("/sync/connect", post(Self::sync_connect))
            .route("/sync/pull", post(Self::sync_pull))
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
        request: Result<Json<SyncConnectRequest>, JsonRejection>,
    ) -> Result<Json<SyncConnectResponse>, AppError> {
        let request = parse_json_request("/sync/connect", request)?;
        validate_received_session(&state, &request.current_session)?;
        state
            .push_notification_client
            .set_device_token(request.device_token);
        state.pairing_state.mark_paired();
        Ok(Json(SyncConnectResponse {
            ok: true,
            server_time: Utc::now(),
        }))
    }

    async fn sync_pull(
        State(state): State<AppState>,
        request: Result<Json<SyncPullRequest>, JsonRejection>,
    ) -> Result<Json<SyncPullResponse>, AppError> {
        let request = parse_json_request("/sync/pull", request)?;
        respond_with_refresh(&state, state.service.pull(request))
    }

    async fn sync_push(
        State(state): State<AppState>,
        request: Result<Json<SyncPushRequest>, JsonRejection>,
    ) -> Result<Json<SyncPushResponse>, AppError> {
        let request = parse_json_request("/sync/push", request)?;
        validate_received_session(&state, &request.current_session)?;
        respond_with_refresh(&state, state.service.push(request))
    }

    async fn sync_ack(
        State(state): State<AppState>,
        request: Result<Json<SyncAckRequest>, JsonRejection>,
    ) -> Result<Json<SyncAckResponse>, AppError> {
        let request = parse_json_request("/sync/ack", request)?;
        respond_with_refresh(&state, state.service.ack(request))
    }
}

fn validate_received_session(
    state: &AppState,
    current_session: &Option<CurrentSession>,
) -> Result<(), AppError> {
    state
        .service
        .validate_received_session(current_session)
        .map_err(AppError::conflict)
}

fn notify_content_changed(content_refresh_tx: &watch::Sender<u64>) {
    let next_version = (*content_refresh_tx.borrow()).wrapping_add(1);
    let _ = content_refresh_tx.send(next_version);
}

fn discover_local_ip() -> anyhow::Result<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip())
}

fn parse_json_request<T>(
    route: &'static str,
    request: Result<Json<T>, JsonRejection>,
) -> Result<T, AppError> {
    match request {
        Ok(Json(request)) => Ok(request),
        Err(rejection) => {
            let status = rejection.status();
            let error = rejection.body_text();
            eprintln!(
                "json rejection on {route}: status={} error={error}",
                status.as_u16()
            );
            Err(AppError::json(status, error))
        }
    }
}

fn respond_with_refresh<T>(
    state: &AppState,
    result: anyhow::Result<T>,
) -> Result<Json<T>, AppError> {
    let response = result.map_err(AppError::internal)?;
    notify_content_changed(&state.content_refresh_tx);
    Ok(Json(response))
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    error: String,
}

impl AppError {
    fn json(status: StatusCode, error: String) -> Self {
        Self { status, error }
    }

    fn conflict(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            error: error.to_string(),
        }
    }

    fn internal(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: error.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse { error: self.error })).into_response()
    }
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
