use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database;
use crate::models::{Check, Comment, Tag};

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8787";

pub async fn spawn() -> anyhow::Result<()> {
    let state = AppState {
        service: Arc::new(SyncService::new(database::database_path())),
    };
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr()?).await?;

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

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/sync/pull", get(sync_pull))
        .route("/sync/push", post(sync_push))
        .route("/sync/ack", post(sync_ack))
        .with_state(state)
}

#[derive(Clone)]
struct AppState {
    service: Arc<SyncService>,
}

#[derive(Debug, Clone)]
struct SyncService {
    database_path: PathBuf,
}

impl SyncService {
    fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    fn pull(&self, limit: Option<usize>) -> anyhow::Result<SyncPullResponse> {
        let connection = database::establish_connection_at(self.database_path.clone())?;

        Ok(SyncPullResponse {
            checks: database::checks::fetch_unsent(&connection, limit)?,
            comments: database::comments::fetch_unsent(&connection, limit)?,
            tags: database::tags::fetch_unsent(&connection, limit)?,
            server_time: Utc::now(),
        })
    }

    fn push(&self, request: SyncPushRequest) -> anyhow::Result<SyncPushResponse> {
        let connection = database::establish_connection_at(self.database_path.clone())?;

        let mut checks_upserted = 0;
        for mut check in request.checks {
            check.is_sent = true;
            database::checks::upsert(&connection, &check)?;
            checks_upserted += 1;
        }

        let mut comments_upserted = 0;
        for mut comment in request.comments {
            comment.is_sent = true;
            database::comments::upsert(&connection, &comment)?;
            comments_upserted += 1;
        }

        let mut tags_upserted = 0;
        for mut tag in request.tags {
            tag.is_sent = true;
            database::tags::upsert(&connection, &tag)?;
            tags_upserted += 1;
        }

        Ok(SyncPushResponse {
            checks_upserted,
            comments_upserted,
            tags_upserted,
            server_time: Utc::now(),
        })
    }

    fn ack(&self, request: SyncAckRequest) -> anyhow::Result<SyncAckResponse> {
        let connection = database::establish_connection_at(self.database_path.clone())?;

        Ok(SyncAckResponse {
            checks_marked_sent: database::checks::mark_sent_by_uuids(&connection, &request.checks)?,
            comments_marked_sent: database::comments::mark_sent_by_uuids(
                &connection,
                &request.comments,
            )?,
            tags_marked_sent: database::tags::mark_sent_by_uuids(&connection, &request.tags)?,
            server_time: Utc::now(),
        })
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    server_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct SyncPullQuery {
    #[allow(dead_code)]
    device_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SyncPushRequest {
    #[allow(dead_code)]
    device_id: Option<String>,
    checks: Vec<Check>,
    comments: Vec<Comment>,
    tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncPushResponse {
    checks_upserted: usize,
    comments_upserted: usize,
    tags_upserted: usize,
    server_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SyncAckRequest {
    #[allow(dead_code)]
    device_id: Option<String>,
    checks: Vec<Uuid>,
    comments: Vec<Uuid>,
    tags: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncAckResponse {
    checks_marked_sent: usize,
    comments_marked_sent: usize,
    tags_marked_sent: usize,
    server_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncPullResponse {
    checks: Vec<Check>,
    comments: Vec<Comment>,
    tags: Vec<Tag>,
    server_time: DateTime<Utc>,
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

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{SyncAckRequest, SyncPushRequest, SyncService};
    use crate::database;
    use crate::models::{Check, Comment, CommentType, Tag};

    #[test]
    fn push_pull_and_ack_round_trip() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());

        let connection = database::establish_connection_at(db_path)?;
        let mut local_check = Check::new("Scout");
        local_check.is_sent = false;
        database::checks::insert(&connection, &local_check)?;

        let pull_before = service.pull(None)?;
        assert_eq!(pull_before.checks.len(), 1);
        assert_eq!(pull_before.checks[0].uuid, local_check.uuid);

        let ack_response = service.ack(SyncAckRequest {
            checks: vec![local_check.uuid],
            comments: vec![],
            tags: vec![],
            device_id: None,
        })?;
        assert_eq!(ack_response.checks_marked_sent, 1);
        assert!(service.pull(None)?.checks.is_empty());

        let remote_comment = Comment::new(CommentType::Game, "Synced from iPhone");
        let remote_tag = Tag::new("Defense", "#000000", "#FFFFFF");
        let push_response = service.push(SyncPushRequest {
            device_id: None,
            checks: vec![],
            comments: vec![remote_comment.clone()],
            tags: vec![remote_tag.clone()],
        })?;
        assert_eq!(push_response.comments_upserted, 1);
        assert_eq!(push_response.tags_upserted, 1);

        let comments = database::comments::fetch_all(&connection)?;
        let tags = database::tags::fetch_all(&connection)?;
        assert_eq!(comments.len(), 1);
        assert!(comments[0].is_sent);
        assert_eq!(tags.len(), 1);
        assert!(tags[0].is_sent);

        Ok(())
    }
}
