// This module wires the server submodules together and exposes the public entrypoint.
mod dto;
mod http;
mod notifications;
mod pairing;
mod service;

pub use http::{spawn, ServerConnectionInfo};
pub use notifications::PushNotificationClient;
pub use pairing::PairingState;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::dto::{SyncAckRequest, SyncPushRequest};
    use super::service::SyncService;
    use crate::database;
    use crate::models::{Check, Comment, CommentType, Tag};

    #[test]
    fn push_pull_and_ack_round_trip() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());

        let connection = database::establish_connection_at(&db_path)?;
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

    #[test]
    fn push_deletes_missing_sent_records_but_keeps_missing_unsent_records() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());
        let connection = database::establish_connection_at(&db_path)?;

        let mut sent_check = Check::new("Delete sent check");
        sent_check.is_sent = true;
        database::checks::insert(&connection, &sent_check)?;

        let unsent_check = Check::new("Keep unsent check");
        database::checks::insert(&connection, &unsent_check)?;

        let mut sent_comment = Comment::new(CommentType::Game, "Delete sent comment");
        sent_comment.is_sent = true;
        database::comments::insert(&connection, &sent_comment)?;

        let unsent_comment = Comment::new(CommentType::Turn, "Keep unsent comment");
        database::comments::insert(&connection, &unsent_comment)?;

        let mut sent_tag = Tag::new("Delete sent tag", "#111111", "#FFFFFF");
        sent_tag.is_sent = true;
        database::tags::insert(&connection, &sent_tag)?;

        let unsent_tag = Tag::new("Keep unsent tag", "#222222", "#FFFFFF");
        database::tags::insert(&connection, &unsent_tag)?;

        let pushed_check = Check::new("Remote check");
        let pushed_comment = Comment::new(CommentType::Game, "Remote comment");
        let pushed_tag = Tag::new("Remote tag", "#333333", "#FFFFFF");

        service.push(SyncPushRequest {
            device_id: None,
            checks: vec![pushed_check.clone()],
            comments: vec![pushed_comment.clone()],
            tags: vec![pushed_tag.clone()],
        })?;

        assert!(database::checks::fetch_by_uuid(&connection, &sent_check.uuid)?.is_none());
        let kept_unsent_check = database::checks::fetch_by_uuid(&connection, &unsent_check.uuid)?
            .expect("unsent check exists");
        assert!(!kept_unsent_check.is_sent);
        let synced_check = database::checks::fetch_by_uuid(&connection, &pushed_check.uuid)?
            .expect("pushed check exists");
        assert!(synced_check.is_sent);

        assert!(database::comments::fetch_by_uuid(&connection, &sent_comment.uuid)?.is_none());
        let kept_unsent_comment =
            database::comments::fetch_by_uuid(&connection, &unsent_comment.uuid)?
                .expect("unsent comment exists");
        assert!(!kept_unsent_comment.is_sent);
        let synced_comment = database::comments::fetch_by_uuid(&connection, &pushed_comment.uuid)?
            .expect("pushed comment exists");
        assert!(synced_comment.is_sent);

        assert!(database::tags::fetch_by_uuid(&connection, &sent_tag.uuid)?.is_none());
        let kept_unsent_tag = database::tags::fetch_by_uuid(&connection, &unsent_tag.uuid)?
            .expect("unsent tag exists");
        assert!(!kept_unsent_tag.is_sent);
        let synced_tag = database::tags::fetch_by_uuid(&connection, &pushed_tag.uuid)?
            .expect("pushed tag exists");
        assert!(synced_tag.is_sent);

        Ok(())
    }
}
