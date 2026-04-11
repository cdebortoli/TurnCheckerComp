use super::NewCheckDraft;
use crate::models::CheckRepeatType;
use uuid::Uuid;

#[test]
fn draft_builds_everytime_check() {
    let draft = NewCheckDraft {
        name: "Scout".to_string(),
        ..Default::default()
    };

    let check = draft.to_check().expect("draft should convert");
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

    let error = draft.to_check().expect_err("repeat value should fail");
    assert!(error.contains("at least 1"));
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

    let check = draft.to_check().expect("draft should convert");
    assert_eq!(check.repeat_case, CheckRepeatType::Specific(4));
    assert_eq!(check.tag_uuid, Some(tag_uuid));
}
