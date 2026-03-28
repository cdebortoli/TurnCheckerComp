// This file contains the database-backed sync business logic used by the HTTP layer.
use std::path::PathBuf;

use chrono::Utc;

use crate::database;

use super::dto::{SyncAckRequest, SyncAckResponse, SyncPullResponse, SyncPushRequest, SyncPushResponse};

#[derive(Debug, Clone)]
pub(super) struct SyncService {
    database_path: PathBuf,
}

impl SyncService {
    pub(super) fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    pub(super) fn pull(&self, limit: Option<usize>) -> anyhow::Result<SyncPullResponse> {
        let connection = database::establish_connection_at(self.database_path.clone())?;

        Ok(SyncPullResponse {
            checks: database::checks::fetch_unsent(&connection, limit)?,
            comments: database::comments::fetch_unsent(&connection, limit)?,
            tags: database::tags::fetch_unsent(&connection, limit)?,
            server_time: Utc::now(),
        })
    }

    pub(super) fn push(&self, request: SyncPushRequest) -> anyhow::Result<SyncPushResponse> {
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

    pub(super) fn ack(&self, request: SyncAckRequest) -> anyhow::Result<SyncAckResponse> {
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
