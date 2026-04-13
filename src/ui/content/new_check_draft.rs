use crate::i18n::{I18n, I18nValue};
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
    pub(super) fn to_check(&self, i18n: &I18n) -> Result<Check, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(i18n.t("validation-name-required"));
        }

        let repeat_field_name = i18n.t("field-repeat-value");
        let repeat_case = match self.repeat_case {
            CheckRepeatType::Everytime => CheckRepeatType::Everytime,
            CheckRepeatType::Conditional(_) => CheckRepeatType::Conditional(parse_positive_i32(
                &self.repeat_value,
                &repeat_field_name,
                i18n,
            )?),
            CheckRepeatType::Specific(_) => CheckRepeatType::Specific(parse_positive_i32(
                &self.repeat_value,
                &repeat_field_name,
                i18n,
            )?),
            CheckRepeatType::Until(_) => CheckRepeatType::Until(parse_positive_i32(
                &self.repeat_value,
                &repeat_field_name,
                i18n,
            )?),
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

fn parse_positive_i32(value: &str, field_name: &str, i18n: &I18n) -> Result<i32, String> {
    let parsed = value.trim().parse::<i32>().map_err(|_| {
        i18n.tr(
            "validation-field-valid-integer",
            &[("field", I18nValue::from(field_name))],
        )
    })?;

    if parsed < 1 {
        return Err(i18n.tr(
            "validation-field-at-least",
            &[
                ("field", I18nValue::from(field_name)),
                ("min", I18nValue::from(1_i32)),
            ],
        ));
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
#[path = "new_check_draft_tests.rs"]
mod tests;
