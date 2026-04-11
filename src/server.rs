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
    use uuid::Uuid;

    use super::dto::{SyncAckRequest, SyncPullRequest, SyncPushRequest};
    use super::service::SyncService;
    use crate::database;
    use crate::models::{Check, Comment, CommentType, CurrentSession, Tag};

    #[test]
    fn pull_and_ack_round_trip_always_includes_current_session() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());

        let connection = database::establish_connection_at(&db_path)?;
        let mut local_check = Check::new("Scout");
        local_check.is_sent = false;
        database::checks::insert(&connection, &local_check)?;
        let local_session = CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 9);
        let local_session_cloned = local_session.clone();
        database::current_session::upsert(&connection, &local_session)?;

        let pull_before = service.pull(SyncPullRequest {
            device_id: None,
            current_session: Some(local_session),
        })?;
        assert_eq!(pull_before.checks.len(), 1);
        assert_eq!(pull_before.checks[0].uuid, local_check.uuid);
        let pulled_session = pull_before
            .current_session
            .expect("session should be pulled");
        assert_eq!(pulled_session.game_name, "Civ VI");
        assert_eq!(pulled_session.turn_number, 9);

        let ack_response = service.ack(SyncAckRequest {
            checks: vec![local_check.uuid],
            comments: vec![],
            tags: vec![],
            device_id: None,
            current_session: None,
        })?;
        assert_eq!(ack_response.checks_marked_sent, 1);
        let pull_after_ack = service.pull(SyncPullRequest {
            device_id: None,
            current_session: Some(local_session_cloned),
        })?;
        assert!(pull_after_ack.checks.is_empty());
        let pulled_session = pull_after_ack
            .current_session
            .expect("session should still always be pulled");
        assert_eq!(pulled_session.game_name, "Civ VI");
        assert_eq!(pulled_session.turn_number, 9);

        let stored_session =
            database::current_session::fetch(&connection)?.expect("session exists");
        assert_eq!(stored_session.game_name, "Civ VI");
        assert_eq!(stored_session.turn_number, 9);

        Ok(())
    }

    #[test]
    fn push_upserts_current_session_without_send_state() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());
        let connection = database::establish_connection_at(&db_path)?;

        let remote_comment = Comment::new(CommentType::Game, "Synced from iPhone");
        let remote_tag = Tag::new("Defense", "#000000", "#FFFFFF");
        let game_uuid = Uuid::new_v4();
        let remote_session = CurrentSession::new(Some(game_uuid), "Civ VII", 3);
        let push_response = service.push(SyncPushRequest {
            device_id: None,
            checks: vec![],
            comments: vec![remote_comment.clone()],
            tags: vec![remote_tag.clone()],
            current_session: Some(remote_session.clone()),
        })?;
        assert_eq!(push_response.comments_upserted, 1);
        assert_eq!(push_response.tags_upserted, 1);
        assert_eq!(push_response.current_session_upserted, 1);

        let comments = database::comments::fetch_all(&connection)?;
        let tags = database::tags::fetch_all(&connection)?;
        let session = database::current_session::fetch(&connection)?.expect("session exists");
        assert_eq!(comments.len(), 1);
        assert!(comments[0].is_sent);
        assert_eq!(tags.len(), 1);
        assert!(tags[0].is_sent);
        assert_eq!(session.game_uuid, Some(game_uuid));
        assert_eq!(session.game_name, "Civ VII");
        assert_eq!(session.turn_number, 3);

        Ok(())
    }

    // #[test]
    // fn validate_received_game_uuid_rejects_mismatch() -> Result<()> {
    //     let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
    //     std::fs::create_dir_all(&temp_dir)?;
    //     let db_path = temp_dir.join("sync.db");
    //     let service = SyncService::new(db_path.clone());
    //     let connection = database::establish_connection_at(&db_path)?;

    //     database::current_session::upsert(
    //         &connection,
    //         &CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 10),
    //     )?;

    //     let error = service
    //         .validate_received_game_uuid(Some(Uuid::new_v4()))
    //         .expect_err("mismatch should fail");
    //     assert!(error.to_string().contains("game uuid mismatch"));

    //     Ok(())
    // }

    #[test]
    fn push_rejects_mismatched_game_uuid() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("turn-checker-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("sync.db");
        let service = SyncService::new(db_path.clone());
        let connection = database::establish_connection_at(&db_path)?;

        let stored_game_uuid = Uuid::new_v4();
        database::current_session::upsert(
            &connection,
            &CurrentSession::new(Some(stored_game_uuid), "Stored Game", 2),
        )?;

        let error = service
            .push(SyncPushRequest {
                device_id: None,
                checks: vec![],
                comments: vec![],
                tags: vec![],
                current_session: Some(CurrentSession::new(Some(Uuid::new_v4()), "Other Game", 4)),
            })
            .expect_err("push should fail on mismatch");
        assert!(error.to_string().contains("game uuid mismatch"));

        Ok(())
    }
}
