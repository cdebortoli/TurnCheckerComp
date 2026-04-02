use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType};
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct NewCheckDraft {
    pub(super) name: String,
    pub(super) detail: String,
    pub(super) selected_tag_uuid: Option<Uuid>,
    pub(super) source: CheckSourceType,
    pub(super) repeat_case: CheckRepeatType,
    pub(super) repeat_value: String,
    pub(super) is_mandatory: bool,
    pub(super) is_checked: bool,
}

impl Default for NewCheckDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            detail: String::new(),
            selected_tag_uuid: None,
            source: CheckSourceType::Game,
            repeat_case: CheckRepeatType::Everytime,
            repeat_value: String::new(),
            is_mandatory: false,
            is_checked: false,
        }
    }
}

impl NewCheckDraft {
    pub(super) fn to_check(&self) -> Result<Check, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Name is required.".to_string());
        }

        let repeat_case = match self.repeat_case {
            CheckRepeatType::Everytime => CheckRepeatType::Everytime,
            CheckRepeatType::Conditional(_) => CheckRepeatType::Conditional(parse_positive_i32(
                &self.repeat_value,
                "Repeat value",
            )?),
            CheckRepeatType::Specific(_) => {
                CheckRepeatType::Specific(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
            CheckRepeatType::Until(_) => {
                CheckRepeatType::Until(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
        };

        let mut check = Check::new(name);
        check.detail = trimmed_option(&self.detail);
        check.tag_uuid = self.selected_tag_uuid;
        check.source = self.source.clone();
        check.repeat_case = repeat_case;
        check.is_mandatory = self.is_mandatory;
        check.is_checked = self.is_checked;
        check.is_sent = false;
        Ok(check)
    }
}

fn parse_positive_i32(value: &str, field_name: &str) -> Result<i32, String> {
    let parsed = value
        .trim()
        .parse::<i32>()
        .map_err(|_| format!("{field_name} must be a valid integer."))?;

    if parsed < 1 {
        return Err(format!("{field_name} must be at least 1."));
    }

    Ok(parsed)
}

fn trimmed_option(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
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
}
