mod checklist;
mod comments;
mod main;
mod new_check;
mod toggle_button;

use crate::database;
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, Tag};
use eframe::egui;
use tokio::sync::watch;
use uuid::Uuid;

pub struct MainContentView {
    mode: ContentMode,
    checks: Vec<Check>,
    tags: Vec<Tag>,
    new_check_draft: NewCheckDraft,
    error_message: Option<String>,
    restart_confirmation_unsent_checks: Option<usize>,
    needs_reload: bool,
    content_refresh_rx: watch::Receiver<u64>,
}

pub enum ContentAction {
    RestartRequested,
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
    selected_tag_uuid: Option<Uuid>,
    source: CheckSourceType,
    repeat_case: CheckRepeatType,
    repeat_value: String,
    is_mandatory: bool,
    is_checked: bool,
}

impl MainContentView {
    pub fn new(content_refresh_rx: watch::Receiver<u64>) -> Self {
        Self {
            mode: ContentMode::General,
            checks: Vec::new(),
            tags: Vec::new(),
            new_check_draft: NewCheckDraft::default(),
            error_message: None,
            restart_confirmation_unsent_checks: None,
            needs_reload: true,
            content_refresh_rx,
        }
    }

    pub fn set_error_message(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    pub fn prepare_for_restart(&mut self) {
        self.mode = ContentMode::General;
        self.checks.clear();
        self.tags.clear();
        self.new_check_draft = NewCheckDraft::default();
        self.error_message = None;
        self.restart_confirmation_unsent_checks = None;
        self.needs_reload = true;
    }

    fn sync_external_content_updates(&mut self) {
        match self.content_refresh_rx.has_changed() {
            Ok(true) => {
                self.content_refresh_rx.borrow_and_update();
                self.needs_reload = true;
            }
            Ok(false) | Err(_) => {}
        }
    }

    fn reload_checks_if_needed(&mut self) {
        if !self.needs_reload {
            return;
        }

        match Self::load_content() {
            Ok((checks, tags)) => {
                self.checks = checks;
                self.tags = tags;
                self.error_message = None;
            }
            Err(error) => self.error_message = Some(error),
        }
        self.needs_reload = false;
    }

    fn load_content() -> Result<(Vec<Check>, Vec<Tag>), String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        let checks = database::checks::fetch_all(&connection).map_err(|err| err.to_string())?;
        let tags = database::tags::fetch_all(&connection).map_err(|err| err.to_string())?;
        Ok((checks, tags))
    }

    fn update_check_status(&mut self, mut check: Check, is_checked: bool) -> Result<(), String> {
        check = apply_check_status_update(check, is_checked);
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

    fn handle_restart_click(&mut self) -> Option<ContentAction> {
        match self.count_unsent_checks() {
            Ok(0) => Some(ContentAction::RestartRequested),
            Ok(unsent_checks) => {
                self.restart_confirmation_unsent_checks = Some(unsent_checks);
                None
            }
            Err(error) => {
                self.error_message = Some(error);
                None
            }
        }
    }

    fn count_unsent_checks(&self) -> Result<usize, String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::count_unsent(&connection).map_err(|err| err.to_string())
    }
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
    fn to_check(&self) -> Result<Check, String> {
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

fn apply_check_status_update(mut check: Check, is_checked: bool) -> Check {
    check.is_checked = is_checked;
    check.is_sent = false;
    check
}

fn find_tag_by_uuid<'a>(tags: &'a [Tag], tag_uuid: Option<Uuid>) -> Option<&'a Tag> {
    let tag_uuid = tag_uuid?;
    tags.iter().find(|tag| tag.uuid == tag_uuid)
}

fn parse_hex_color(value: &str) -> Option<egui::Color32> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(egui::Color32::from_rgb(red, green, blue))
}

fn tag_fill_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.color).unwrap_or_else(|| egui::Color32::from_rgb(99, 99, 102))
}

fn tag_text_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.text_color).unwrap_or(egui::Color32::WHITE)
}

fn show_tag_capsule(ui: &mut egui::Ui, tag: &Tag) {
    egui::Frame::new()
        .fill(tag_fill_color(tag))
        .corner_radius(999.0)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&tag.name)
                    .family(egui::FontFamily::Name("montserrat-bold".into()))
                    .color(tag_text_color(tag))
                    .small(),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::{apply_check_status_update, MainContentView, NewCheckDraft};
    use crate::models::Check;
    use crate::models::CheckRepeatType;
    use tokio::sync::watch;
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
}
