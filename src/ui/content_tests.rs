use super::{apply_check_status_update, ContentMode, MainContentView};
use crate::models::{Check, CurrentSession};
use tokio::sync::watch;
use uuid::Uuid;

#[test]
fn external_refresh_marks_content_dirty() {
    let (content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx);
    view.needs_reload = false;

    content_refresh_tx
        .send(1)
        .expect("refresh signal should send");
    view.sync_external_content_updates();

    assert!(view.needs_reload);
    assert!(!view.content_refresh_rx.has_changed().unwrap());
}

#[test]
fn local_status_update_marks_check_unsent() {
    let mut check = Check::new("Scout");
    check.is_sent = true;

    let updated = apply_check_status_update(check, true);

    assert!(updated.is_checked);
    assert!(!updated.is_sent);
}

#[test]
fn next_turn_wait_unlocks_after_turn_increase_for_same_game() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let game_uuid = Uuid::new_v4();
    let mut view = MainContentView::new(content_refresh_rx);
    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));

    view.start_next_turn_wait().expect("wait should start");
    assert!(view.is_waiting_for_next_turn());
    assert_eq!(view.mode, ContentMode::WaitingForNextTurn);

    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 6));
    view.try_finish_next_turn_wait();

    assert!(!view.is_waiting_for_next_turn());
    assert_eq!(view.mode, ContentMode::General);
}

#[test]
fn next_turn_wait_stays_locked_without_turn_increase() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let game_uuid = Uuid::new_v4();
    let mut view = MainContentView::new(content_refresh_rx);
    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));

    view.start_next_turn_wait().expect("wait should start");
    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));
    view.try_finish_next_turn_wait();

    assert!(view.is_waiting_for_next_turn());
}

#[test]
fn next_turn_wait_stays_locked_for_different_game_uuid() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx);
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

    view.start_next_turn_wait().expect("wait should start");
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 6));
    view.try_finish_next_turn_wait();

    assert!(view.is_waiting_for_next_turn());
}

#[test]
fn cancel_next_turn_wait_clears_wait_state_and_sets_error() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx);
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));
    view.start_next_turn_wait().expect("wait should start");

    view.cancel_next_turn_wait("push failed");

    assert!(!view.is_waiting_for_next_turn());
    assert_eq!(view.mode, ContentMode::General);
    assert_eq!(view.error_message.as_deref(), Some("push failed"));
}
