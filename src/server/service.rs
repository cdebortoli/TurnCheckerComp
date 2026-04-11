// This file contains the database-backed sync business logic used by the HTTP layer.
use std::path::PathBuf;

use chrono::Utc;

use crate::{database, models::CurrentSession, server::dto::SyncPullRequest};

use super::dto::{
    SyncAckRequest, SyncAckResponse, SyncPullResponse, SyncPushRequest, SyncPushResponse,
};

#[derive(Debug, Clone)]
pub(super) struct SyncService {
    database_path: PathBuf,
}

impl SyncService {
    pub(super) fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    pub(super) fn pull(&self, request: SyncPullRequest) -> anyhow::Result<SyncPullResponse> {
        let connection = database::establish_connection_at(&self.database_path)?;

        Ok(SyncPullResponse {
            checks: database::checks::fetch_unsent(&connection)?,
            comments: database::comments::fetch_unsent(&connection)?,
            tags: database::tags::fetch_unsent(&connection)?,
            current_session: database::current_session::fetch(&connection)?,
            server_time: Utc::now(),
        })
    }

    pub(super) fn push(&self, request: SyncPushRequest) -> anyhow::Result<SyncPushResponse> {
        self.validate_push_request(&request)?;

        let connection = database::establish_connection_at(&self.database_path)?;
        let SyncPushRequest {
            device_id: _,
            checks,
            comments,
            tags,
            current_session,
        } = request;

        let check_uuids = checks.iter().map(|check| check.uuid).collect::<Vec<_>>();
        let comment_uuids = comments
            .iter()
            .map(|comment| comment.uuid)
            .collect::<Vec<_>>();
        let tag_uuids = tags.iter().map(|tag| tag.uuid).collect::<Vec<_>>();

        let mut checks_upserted = 0;
        for mut check in checks {
            check.is_sent = true;
            database::checks::upsert(&connection, &check)?;
            checks_upserted += 1;
        }
        database::checks::delete_sent_missing_uuids(&connection, &check_uuids)?;

        let mut comments_upserted = 0;
        for mut comment in comments {
            comment.is_sent = true;
            database::comments::upsert(&connection, &comment)?;
            comments_upserted += 1;
        }
        database::comments::delete_sent_missing_uuids(&connection, &comment_uuids)?;

        let mut tags_upserted = 0;
        for mut tag in tags {
            tag.is_sent = true;
            database::tags::upsert(&connection, &tag)?;
            tags_upserted += 1;
        }
        database::tags::delete_sent_missing_uuids(&connection, &tag_uuids)?;

        let mut current_session_upserted = 0;
        if let Some(current_session) = current_session {
            let database_current_session =
                database::current_session::fetch(&connection).unwrap_or(None);
            match database_current_session {
                Some(db_current_session) => {
                    // Don't update if pushed by ios app while new turn still not processed by ios app
                    if current_session.turn_number >= db_current_session.new_turn_number {
                        database::current_session::upsert(&connection, &current_session)?;
                        current_session_upserted = 1;
                    }
                }
                None => {
                    database::current_session::upsert(&connection, &current_session)?;
                    current_session_upserted = 1;
                }
            };
        }

        Ok(SyncPushResponse {
            checks_upserted,
            comments_upserted,
            tags_upserted,
            current_session_upserted,
            server_time: Utc::now(),
        })
    }

    pub(super) fn ack(&self, request: SyncAckRequest) -> anyhow::Result<SyncAckResponse> {
        let connection = database::establish_connection_at(&self.database_path)?;

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

    pub(super) fn validate_received_session(
        &self,
        received_game_session: &Option<CurrentSession>,
    ) -> anyhow::Result<()> {
        let connection = database::establish_connection_at(&self.database_path)?;
        database::current_session::validate_session_match(&connection, received_game_session)
    }

    pub(super) fn validate_push_request(&self, request: &SyncPushRequest) -> anyhow::Result<()> {
        self.validate_received_session(&request.current_session)
    }
}
