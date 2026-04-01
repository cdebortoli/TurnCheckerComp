// This module wires the server submodules together and exposes the public entrypoint.
mod dto;
mod http;
mod pairing;
mod service;

pub use http::{spawn, ServerConnectionInfo};
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
}
