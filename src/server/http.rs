// This file contains the HTTP server bootstrap, routing, and request handlers.
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use tokio::sync::watch;

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

pub async fn spawn(
    pairing_state: PairingState,
    content_refresh_tx: watch::Sender<u64>,
) -> anyhow::Result<ServerConnectionInfo> {
    HttpServer::new(
        Arc::new(SyncService::new(database::database_path())),
        pairing_state,
        content_refresh_tx,
    )
    .spawn()
    .await
}

#[derive(Clone)]
struct AppState {
    service: Arc<SyncService>,
    pairing_state: PairingState,
    content_refresh_tx: watch::Sender<u64>,
}

pub(super) struct HttpServer {
    state: AppState,
}

impl HttpServer {
    fn new(
        service: Arc<SyncService>,
        pairing_state: PairingState,
        content_refresh_tx: watch::Sender<u64>,
    ) -> Self {
        Self {
            state: AppState {
                service,
                pairing_state,
                content_refresh_tx,
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
        request: Result<Json<SyncConnectRequest>, JsonRejection>,
    ) -> Result<Json<SyncConnectResponse>, Response> {
        let _request = parse_json_request("/sync/connect", request)?;
        state.pairing_state.mark_paired();
        Ok(Json(SyncConnectResponse {
            ok: true,
            server_time: Utc::now(),
        }))
    }

    async fn sync_pull(
        State(state): State<AppState>,
        Query(query): Query<SyncPullQuery>,
    ) -> Result<Json<SyncPullResponse>, AppError> {
        Ok(Json(state.service.pull(query.limit)?))
    }

    async fn sync_push(
        State(state): State<AppState>,
        request: Result<Json<SyncPushRequest>, JsonRejection>,
    ) -> Result<Json<SyncPushResponse>, Response> {
        let request = parse_json_request("/sync/push", request)?;
        match state.service.push(request) {
            Ok(response) => {
                notify_content_changed(&state.content_refresh_tx);
                Ok(Json(response))
            }
            Err(error) => Err(AppError::from(error).into_response()),
        }
    }

    async fn sync_ack(
        State(state): State<AppState>,
        request: Result<Json<SyncAckRequest>, JsonRejection>,
    ) -> Result<Json<SyncAckResponse>, Response> {
        let request = parse_json_request("/sync/ack", request)?;
        state
            .service
            .ack(request)
            .map(|response| {
                notify_content_changed(&state.content_refresh_tx);
                Json(response)
            })
            .map_err(|error| AppError::from(error).into_response())
    }
}

fn notify_content_changed(content_refresh_tx: &watch::Sender<u64>) {
    let next_version = (*content_refresh_tx.borrow()).wrapping_add(1);
    let _ = content_refresh_tx.send(next_version);
}

fn parse_json_request<T>(
    route: &'static str,
    request: Result<Json<T>, JsonRejection>,
) -> Result<T, Response> {
    match request {
        Ok(Json(request)) => Ok(request),
        Err(rejection) => {
            let status = rejection.status();
            let error = rejection.body_text();
            eprintln!(
                "json rejection on {route}: status={} error={error}",
                status.as_u16()
            );
            Err((status, Json(ErrorResponse { error })).into_response())
        }
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

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use uuid::Uuid;

    use super::parse_json_request;
    use crate::server::dto::{SyncConnectRequest, SyncPushRequest};

    #[tokio::test]
    async fn json_rejection_includes_unknown_top_level_field() {
        let response = parse_json_request(
            "/sync/connect",
            axum::Json::<SyncConnectRequest>::from_bytes(
                br#"{"deviceId":"ios","unexpected":true}"#,
            ),
        )
        .unwrap_err();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("unknown field `unexpected`"));
    }

    #[tokio::test]
    async fn json_rejection_includes_nested_field_path() {
        let payload = format!(
            concat!(
                "{{",
                "\"deviceId\":\"ios\",",
                "\"checks\":[{{",
                "\"uuid\":\"{}\",",
                "\"name\":\"Scout\",",
                "\"detail\":null,",
                "\"source\":\"Game\",",
                "\"repeatCase\":\"Everytime\",",
                "\"tagUuid\":null,",
                "\"position\":0,",
                "\"isMandatory\":false,",
                "\"isChecked\":false,",
                "\"isSent\":false,",
                "\"unexpected\":true",
                "}}],",
                "\"comments\":[],",
                "\"tags\":[]",
                "}}"
            ),
            Uuid::new_v4()
        );

        let response = parse_json_request(
            "/sync/push",
            axum::Json::<SyncPushRequest>::from_bytes(payload.as_bytes()),
        )
        .unwrap_err();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("checks[0]"));
        assert!(body.contains("unknown field `unexpected`"));
    }
}
