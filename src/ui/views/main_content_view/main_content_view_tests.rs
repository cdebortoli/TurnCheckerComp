use super::super::checklist_view::ChecklistAction;
use super::super::new_check_view::NewCheckAction;
use super::super::source_checks_view::SourceChecksAction;
use super::{ContentMode, MainContentView};
use crate::i18n::I18n;
use crate::models::{
    check_source_type::CheckSourceType, Check, Comment, CommentType, CurrentSession,
};
use crate::ui::ui_helpers::{apply_check_status_update, apply_comment_content_update};
use tokio::sync::watch;
use uuid::Uuid;

fn test_i18n() -> I18n {
    I18n::from_language("en-US")
}

// Ensures external refresh signal marks UI for reload.
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

// Ensures local check status edits clear sent flag.
#[test]
fn local_status_update_marks_check_unsent() {
    let mut check = Check::new("Scout");
    check.is_sent = true;

    let updated = apply_check_status_update(check, true);

    assert!(updated.is_checked);
    assert!(!updated.is_sent);
}

// Ensures local comment edits clear sent flag.
#[test]
fn local_comment_update_marks_comment_unsent() {
    let mut comment = Comment::new(CommentType::Game, "Synced note");
    comment.is_sent = true;

    let updated = apply_comment_content_update(comment, "Edited note");

    assert_eq!(updated.content, "Edited note");
    assert!(!updated.is_sent);
}

// Ensures next-turn click opens confirmation when session exists.
#[test]
fn next_turn_click_opens_confirmation_when_session_is_available() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

    view.handle_new_turn_click();

    assert_eq!(view.new_turn_confirmation_open, Some(0));
    assert!(view.error_message.is_none());
}

// Ensures confirmation counts unchecked mandatory turn checks only.
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

// Ensures missing session blocks next-turn flow with error.
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

// Ensures wait mode exits after same-game turn increases.
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

// Ensures wait mode remains if turn does not increase.
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

// Ensures wait mode remains if game UUID changes.
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

// Ensures cancel exits wait mode and stores error.
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

// Ensures selecting checklist check opens edit/new-check mode.
#[test]
fn selecting_check_from_editable_list_opens_edit_mode() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    let check = Check::new("Scout");

    view.handle_checklist_action(ChecklistAction::CheckSelected(check));

    assert_eq!(view.mode, ContentMode::NewCheck);
    assert!(view.error_message.is_none());
}

// Ensures back returns to source checks from edit mode.
#[test]
fn navigating_back_from_source_check_edit_returns_to_source_checks() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    let check = Check::new("Scout");

    view.handle_source_checks_action(SourceChecksAction::CheckSelected(check));
    view.navigate_back();

    assert_eq!(view.mode, ContentMode::SourceChecks);
    assert!(view.error_message.is_none());
}

// Ensures cancel returns to source checks from edit mode.
#[test]
fn cancelling_source_check_edit_returns_to_source_checks() {
    let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
    let mut view = MainContentView::new(content_refresh_rx, test_i18n());
    let check = Check::new("Scout");

    view.handle_source_checks_action(SourceChecksAction::CheckSelected(check));
    view.handle_new_check_action(NewCheckAction::Cancelled);

    assert_eq!(view.mode, ContentMode::SourceChecks);
    assert!(view.error_message.is_none());
}
