mod checklist;
mod comments;
mod main;
mod new_check;

use crate::database;
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType};

pub struct MainContentView {
    mode: ContentMode,
    checks: Vec<Check>,
    new_check_draft: NewCheckDraft,
    error_message: Option<String>,
    needs_reload: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    General,
    NewCheck,
    Comments,
}

#[derive(Clone)]
struct NewCheckDraft {
    name: String,
    detail: String,
    source: CheckSourceType,
    repeat_case: CheckRepeatType,
    repeat_value: String,
    position: String,
    is_mandatory: bool,
    is_checked: bool,
}

impl MainContentView {
    pub fn new() -> Self {
        Self {
            mode: ContentMode::General,
            checks: Vec::new(),
            new_check_draft: NewCheckDraft::default(),
            error_message: None,
            needs_reload: true,
        }
    }

    fn reload_checks_if_needed(&mut self) {
        if !self.needs_reload {
            return;
        }

        match Self::load_checks() {
            Ok(checks) => {
                self.checks = checks;
                self.error_message = None;
            }
            Err(error) => self.error_message = Some(error),
        }
        self.needs_reload = false;
    }

    fn load_checks() -> Result<Vec<Check>, String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::fetch_all(&connection).map_err(|err| err.to_string())
    }

    fn update_check_status(&mut self, mut check: Check, is_checked: bool) -> Result<(), String> {
        check.is_checked = is_checked;
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::update(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    fn insert_new_check(&mut self) -> Result<(), String> {
        let check = self.new_check_draft.to_check()?;
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::insert(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }
}

impl Default for NewCheckDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            detail: String::new(),
            source: CheckSourceType::Game,
            repeat_case: CheckRepeatType::Everytime,
            repeat_value: String::new(),
            position: "0".to_string(),
            is_mandatory: false,
            is_checked: false,
        }
    }
}

impl NewCheckDraft {
    fn to_check(&self) -> Result<Check, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Name is required.".to_string());
        }

        let position = self
            .position
            .trim()
            .parse::<i32>()
            .map_err(|_| "Position must be a valid integer.".to_string())?;

        let repeat_case = match self.repeat_case {
            CheckRepeatType::Everytime => CheckRepeatType::Everytime,
            CheckRepeatType::Conditional(_) => {
                CheckRepeatType::Conditional(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
            CheckRepeatType::Specific(_) => {
                CheckRepeatType::Specific(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
            CheckRepeatType::Until(_) => {
                CheckRepeatType::Until(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
        };

        let mut check = Check::new(name);
        check.detail = trimmed_option(&self.detail);
        check.source = self.source.clone();
        check.repeat_case = repeat_case;
        check.position = position;
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

    #[test]
    fn draft_builds_everytime_check() {
        let draft = NewCheckDraft {
            name: "Scout".to_string(),
            position: "2".to_string(),
            ..Default::default()
        };

        let check = draft.to_check().expect("draft should convert");
        assert_eq!(check.name, "Scout");
        assert_eq!(check.position, 2);
        assert_eq!(check.repeat_case, CheckRepeatType::Everytime);
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
        let draft = NewCheckDraft {
            name: "Scout".to_string(),
            repeat_case: CheckRepeatType::Specific(1),
            repeat_value: "4".to_string(),
            ..Default::default()
        };

        let check = draft.to_check().expect("draft should convert");
        assert_eq!(check.repeat_case, CheckRepeatType::Specific(4));
    }
}
