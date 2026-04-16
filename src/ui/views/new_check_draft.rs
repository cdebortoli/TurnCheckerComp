use crate::i18n::{I18n, I18nValue};
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, CurrentSession};
use uuid::Uuid;

pub(super) const MAX_REPEAT_VALUE: i32 = 9_999;

#[derive(Clone)]
pub(super) struct NewCheckDraft {
    existing_check: Option<Check>,
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
            existing_check: None,
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
    pub(super) fn from_check(check: &Check) -> Self {
        Self {
            existing_check: Some(check.clone()),
            name: check.name.clone(),
            detail: check.detail.clone().unwrap_or_default(),
            selected_tag_uuid: check.tag_uuid,
            source: check.source,
            repeat_case: check.repeat_case,
            repeat_value: repeat_value_for(&check.repeat_case),
            is_mandatory: check.is_mandatory,
            is_checked: check.is_checked,
        }
    }

    pub(super) fn is_editing(&self) -> bool {
        self.existing_check.is_some()
    }

    pub(super) fn source_is_locked(&self) -> bool {
        self.is_editing()
    }

    pub(super) fn set_source(
        &mut self,
        source: CheckSourceType,
        current_session: Option<&CurrentSession>,
    ) {
        self.source = source;
        self.sync_source_dependent_fields(current_session);
    }

    pub(super) fn sync_source_dependent_fields(
        &mut self,
        current_session: Option<&CurrentSession>,
    ) {
        if !self.turn_repeat_is_locked() {
            return;
        }

        let turn_number = current_session
            .map(|session| session.turn_number)
            .unwrap_or(1);
        self.repeat_case = CheckRepeatType::Specific(turn_number);
        self.repeat_value = turn_number.to_string();
    }

    pub(super) fn turn_repeat_is_locked(&self) -> bool {
        self.source == CheckSourceType::Turn
    }

    pub(super) fn to_check(
        &self,
        i18n: &I18n,
        current_session: Option<&CurrentSession>,
    ) -> Result<Check, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(i18n.t("validation-name-required"));
        }

        let repeat_field_name = i18n.t("field-repeat-value");
        let repeat_case = if self.turn_repeat_is_locked() {
            let current_session =
                current_session.ok_or_else(|| i18n.t("content-error-no-current-session"))?;
            CheckRepeatType::Specific(current_session.turn_number)
        } else {
            match self.repeat_case {
                CheckRepeatType::Everytime => CheckRepeatType::Everytime,
                CheckRepeatType::Conditional(_) => CheckRepeatType::Conditional(
                    parse_positive_i32(&self.repeat_value, &repeat_field_name, i18n)?,
                ),
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
            }
        };

        let mut check = self
            .existing_check
            .clone()
            .unwrap_or_else(|| Check::new(name));
        check.name = name.to_string();
        check.detail = trimmed_option(&self.detail);
        check.tag_uuid = self.selected_tag_uuid;
        check.source = self.source;
        check.repeat_case = repeat_case;
        check.is_mandatory = self.is_mandatory;
        check.is_checked = self.is_checked;
        check.is_sent = false;
        Ok(check)
    }
}

fn repeat_value_for(repeat_case: &CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => String::new(),
        CheckRepeatType::Conditional(value)
        | CheckRepeatType::Specific(value)
        | CheckRepeatType::Until(value) => value.to_string(),
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

    if parsed > MAX_REPEAT_VALUE {
        return Err(i18n.tr(
            "validation-field-at-most",
            &[
                ("field", I18nValue::from(field_name)),
                ("max", I18nValue::from(MAX_REPEAT_VALUE)),
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
