use super::NewCheckDraft;
use crate::i18n::I18n;
use crate::models::{check_source_type::CheckSourceType, CheckRepeatType, CurrentSession};
use uuid::Uuid;

fn test_i18n() -> I18n {
    I18n::from_language("en-US")
}

#[test]
fn draft_builds_everytime_check() {
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        ..Default::default()
    };

    let check = draft
        .to_check(&test_i18n(), None)
        .expect("draft should convert");
    assert_eq!(check.name, "Scout");
    assert_eq!(check.repeat_case, CheckRepeatType::Everytime);
    assert_eq!(check.tag_uuid, None);
}

#[test]
fn draft_requires_positive_repeat_value() {
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        repeat_case: CheckRepeatType::Until(1),
        repeat_value: "0".to_string(),
        ..Default::default()
    };

    let error = draft
        .to_check(&test_i18n(), None)
        .expect_err("repeat value should fail");
    assert!(error.contains("at least"));
}

#[test]
fn draft_builds_non_default_repeat_type() {
    let tag_uuid = Uuid::new_v4();
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        selected_tag_uuid: Some(tag_uuid),
        repeat_case: CheckRepeatType::Specific(1),
        repeat_value: "4".to_string(),
        ..Default::default()
    };

    let check = draft
        .to_check(&test_i18n(), None)
        .expect("draft should convert");
    assert_eq!(check.repeat_case, CheckRepeatType::Specific(4));
    assert_eq!(check.tag_uuid, Some(tag_uuid));
}

#[test]
fn selecting_turn_source_locks_repeat_to_current_turn() {
    let session = CurrentSession::new(None, "Civ VI", 5);
    let mut draft = NewCheckDraft {
        repeat_case: CheckRepeatType::Until(9),
        repeat_value: "9".to_string(),
        ..Default::default()
    };

    draft.set_source(CheckSourceType::Turn, Some(&session));

    assert_eq!(draft.source, CheckSourceType::Turn);
    assert!(draft.turn_repeat_is_locked());
    assert_eq!(draft.repeat_case, CheckRepeatType::Specific(5));
    assert_eq!(draft.repeat_value, "5");
}

#[test]
fn turn_source_builds_specific_repeat_from_current_session() {
    let session = CurrentSession::new(None, "Civ VI", 7);
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        source: CheckSourceType::Turn,
        repeat_case: CheckRepeatType::Until(1),
        repeat_value: "99".to_string(),
        ..Default::default()
    };

    let check = draft
        .to_check(&test_i18n(), Some(&session))
        .expect("draft should convert");

    assert_eq!(check.repeat_case, CheckRepeatType::Specific(7));
}

#[test]
fn turn_source_requires_current_session() {
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        source: CheckSourceType::Turn,
        ..Default::default()
    };

    let error = draft
        .to_check(&test_i18n(), None)
        .expect_err("turn source should require current session");

    assert_eq!(error, "No current session is available yet.");
}
