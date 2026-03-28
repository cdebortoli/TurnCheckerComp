// This file defines the request, query, and response structs exchanged as JSON by the server.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Check, Comment, Tag};

#[derive(Debug, Serialize)]
pub(super) struct HealthResponse {
    pub status: &'static str,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SyncPullQuery {
    #[allow(dead_code)]
    pub device_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyncPushRequest {
    #[allow(dead_code)]
    pub device_id: Option<String>,
    pub checks: Vec<Check>,
    pub comments: Vec<Comment>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyncPushResponse {
    pub checks_upserted: usize,
    pub comments_upserted: usize,
    pub tags_upserted: usize,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyncAckRequest {
    #[allow(dead_code)]
    pub device_id: Option<String>,
    pub checks: Vec<Uuid>,
    pub comments: Vec<Uuid>,
    pub tags: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyncAckResponse {
    pub checks_marked_sent: usize,
    pub comments_marked_sent: usize,
    pub tags_marked_sent: usize,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyncPullResponse {
    pub checks: Vec<Check>,
    pub comments: Vec<Comment>,
    pub tags: Vec<Tag>,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(super) struct ErrorResponse {
    pub error: String,
}
