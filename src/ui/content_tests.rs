use super::checklist::ChecklistAction;
use super::helpers::{apply_check_status_update, apply_comment_content_update};
use super::{ContentMode, MainContentView};
use crate::i18n::I18n;
use crate::models::{
    check_source_type::CheckSourceType, Check, Comment, CommentType, CurrentSession,
};
use tokio::sync::watch;
use uuid::Uuid;

fn test_i18n() -> I18n {
    I18n::from_language("en-US")
}

#[test]
fn external_refresh_marks_content_dirty() {
    let (content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
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
fn local_comment_update_marks_comment_unsent() {
    let mut comment = Comment::new(CommentType::Game, "Synced note");
    comment.is_sent = true;

    let updated = apply_comment_content_update(comment, "Edited note");

    assert_eq!(updated.content, "Edited note");
    assert!(!updated.is_sent);
}

#[test]
fn next_turn_click_opens_confirmation_when_session_is_available() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

    view.handle_new_turn_click();

    assert_eq!(view.new_turn_confirmation_open, Some(0));
    assert!(view.error_message.is_none());
}

#[test]
fn next_turn_click_counts_unchecked_mandatory_turn_checks() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

    let mut unchecked_turn = Check::new("Scout");
    unchecked_turn.source = CheckSourceType::Turn;
    unchecked_turn.is_mandatory = true;

    let mut checked_turn = Check::new("City");
    checked_turn.source = CheckSourceType::Turn;
    checked_turn.is_mandatory = true;
    checked_turn.is_checked = true;

    let mut unchecked_global = Check::new("Global");
    unchecked_global.source = CheckSourceType::GlobalGame;
    unchecked_global.is_mandatory = true;

    view.checks = vec![unchecked_turn, checked_turn, unchecked_global];

    view.handle_new_turn_click();

    assert_eq!(view.new_turn_confirmation_open, Some(1));
}

#[test]
fn next_turn_click_sets_error_when_session_is_missing() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());

    view.handle_new_turn_click();

    assert_eq!(view.new_turn_confirmation_open, None);
    assert_eq!(
        view.error_message.as_deref(),
        Some("No current session is available yet.")
    );
}

#[test]
fn next_turn_wait_unlocks_after_turn_increase_for_same_game() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let game_uuid = Uuid::new_v4();
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
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
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));

    view.start_next_turn_wait().expect("wait should start");
    view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));
    view.try_finish_next_turn_wait();

    assert!(view.is_waiting_for_next_turn());
}

#[test]
fn next_turn_wait_stays_locked_for_different_game_uuid() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

    view.start_next_turn_wait().expect("wait should start");
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 6));
    view.try_finish_next_turn_wait();

    assert!(view.is_waiting_for_next_turn());
}

#[test]
fn cancel_next_turn_wait_clears_wait_state_and_sets_error() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));
    view.start_next_turn_wait().expect("wait should start");

    view.cancel_next_turn_wait("push failed");

    assert!(!view.is_waiting_for_next_turn());
    assert_eq!(view.mode, ContentMode::General);
    assert_eq!(view.error_message.as_deref(), Some("push failed"));
}

#[test]
fn selecting_check_from_editable_list_opens_edit_mode() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    let check = Check::new("Scout");

    view.handle_checklist_action(ChecklistAction::CheckSelected(check));

    assert_eq!(view.mode, ContentMode::NewCheck);
    assert!(view.error_message.is_none());
}
