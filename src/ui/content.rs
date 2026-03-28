use eframe::egui::{self, RichText};

use crate::database;
use crate::models::{Check, CheckRepeatType};
use crate::models::check_source_type::CheckSourceType;

use super::theme::Theme;

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
    repeat_kind: RepeatKind,
    repeat_value: String,
    position: String,
    is_mandatory: bool,
    is_checked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RepeatKind {
    Everytime,
    Conditional,
    Specific,
    Until,
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

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::from_visuals(ui.visuals());
        self.reload_checks_if_needed();

        egui::Frame::new()
            .fill(theme.bg_primary)
            .inner_margin(theme.spacing_lg)
            .show(ui, |ui| {
                self.show_top_bar(ui, &theme);
                ui.add_space(theme.spacing_md);

                if let Some(error) = &self.error_message {
                    ui.label(RichText::new(error).color(theme.destructive));
                    ui.add_space(theme.spacing_md);
                }

                match self.mode {
                    ContentMode::General => self.show_general_content(ui, &theme),
                    ContentMode::NewCheck => self.show_new_check_content(ui, &theme),
                    ContentMode::Comments => self.show_comments_content(ui, &theme),
                }
            });
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Checks").color(theme.text_primary));
            ui.add_space(theme.spacing_md);

            let new_check = egui::Button::new(
                RichText::new("New Check").color(theme.text_primary),
            )
            .fill(if self.mode == ContentMode::NewCheck {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);
            if ui.add(new_check).clicked() {
                self.mode = ContentMode::NewCheck;
                self.error_message = None;
            }

            let comments = egui::Button::new(
                RichText::new("Comments").color(theme.text_primary),
            )
            .fill(if self.mode == ContentMode::Comments {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);
            if ui.add(comments).clicked() {
                self.mode = ContentMode::Comments;
                self.error_message = None;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.mode != ContentMode::General
                    && ui.button(RichText::new("Back").color(theme.text_primary)).clicked()
                {
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
            });
        });
    }

    fn show_general_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("All checks").color(theme.text_secondary));
        ui.add_space(theme.spacing_md);

        if self.checks.is_empty() {
            ui.label(RichText::new("No checks yet.").color(theme.text_muted));
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for index in 0..self.checks.len() {
                    let check_snapshot = self.checks[index].clone();
                    let mut selected_checked = check_snapshot.is_checked;

                    egui::Frame::new()
                        .fill(theme.bg_list_element)
                        .corner_radius(theme.corner_radius)
                        .inner_margin(theme.card_padding)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(4.0, 36.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter()
                                    .rect_filled(rect, theme.corner_radius, source_color(&check_snapshot, theme));

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(&check_snapshot.name)
                                                .color(theme.text_primary)
                                                .strong(),
                                        );
                                        if check_snapshot.is_mandatory {
                                            ui.label(
                                                RichText::new("Mandatory")
                                                    .color(theme.warning)
                                                    .small(),
                                            );
                                        }
                                        status_badge(ui, theme, &check_snapshot.repeat_case);
                                    });

                                    if let Some(detail) = &check_snapshot.detail {
                                        if !detail.is_empty() {
                                            ui.label(
                                                RichText::new(detail).color(theme.text_secondary),
                                            );
                                        }
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        egui::ComboBox::from_id_salt(("check_status", check_snapshot.id))
                                            .selected_text(if selected_checked {
                                                "Checked"
                                            } else {
                                                "Unchecked"
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut selected_checked,
                                                    false,
                                                    "Unchecked",
                                                );
                                                ui.selectable_value(
                                                    &mut selected_checked,
                                                    true,
                                                    "Checked",
                                                );
                                            });
                                    },
                                );
                            });

                            ui.add_space(theme.spacing_sm);
                            ui.separator();
                            ui.add_space(theme.spacing_sm);

                            let repeat_text = repeat_label(&check_snapshot.repeat_case);
                            let position_text = check_snapshot.position.to_string();
                            let uuid_text = check_snapshot.uuid.to_string();

                            egui::Grid::new(("check_grid", check_snapshot.id))
                                .num_columns(2)
                                .spacing([theme.spacing_lg, theme.spacing_sm])
                                .show(ui, |ui| {
                                    field_row(ui, theme, "Source", source_label(&check_snapshot.source));
                                    field_row(ui, theme, "Repeat", &repeat_text);
                                    field_row(ui, theme, "Position", &position_text);
                                    field_row(
                                        ui,
                                        theme,
                                        "Selected",
                                        if check_snapshot.is_checked { "Checked" } else { "Unchecked" },
                                    );
                                    field_row(
                                        ui,
                                        theme,
                                        "Mandatory",
                                        if check_snapshot.is_mandatory { "Yes" } else { "No" },
                                    );
                                    field_row(
                                        ui,
                                        theme,
                                        "Sent",
                                        if check_snapshot.is_sent { "Yes" } else { "No" },
                                    );
                                    field_row(ui, theme, "UUID", &uuid_text);
                                });
                        });

                    if selected_checked != check_snapshot.is_checked {
                        if let Err(error) = self.update_check_status(check_snapshot, selected_checked) {
                            self.error_message = Some(error);
                        }
                    }

                    ui.add_space(theme.spacing_md);
                }
            });
    }

    fn show_new_check_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading(RichText::new("Create a new check").color(theme.text_primary));
        ui.label(RichText::new("Set all parameters before saving.").color(theme.text_secondary));
        ui.add_space(theme.spacing_md);

        egui::ScrollArea::vertical().show(ui, |ui| {
            labeled_text_edit(ui, theme, "Name", &mut self.new_check_draft.name, false);
            labeled_text_edit(ui, theme, "Detail", &mut self.new_check_draft.detail, true);

            ui.label(RichText::new("Source").color(theme.text_secondary));
            egui::ComboBox::from_id_salt("new_check_source")
                .selected_text(source_label(&self.new_check_draft.source))
                .show_ui(ui, |ui| {
                    for source in [
                        CheckSourceType::Game,
                        CheckSourceType::GlobalGame,
                        CheckSourceType::Blueprint,
                        CheckSourceType::Turn,
                    ] {
                        ui.selectable_value(
                            &mut self.new_check_draft.source,
                            source.clone(),
                            source_label(&source),
                        );
                    }
                });
            ui.add_space(theme.spacing_md);

            ui.label(RichText::new("Repeat").color(theme.text_secondary));
            egui::ComboBox::from_id_salt("new_check_repeat_kind")
                .selected_text(self.new_check_draft.repeat_kind.label())
                .show_ui(ui, |ui| {
                    for kind in [
                        RepeatKind::Everytime,
                        RepeatKind::Conditional,
                        RepeatKind::Specific,
                        RepeatKind::Until,
                    ] {
                        if ui
                            .selectable_label(self.new_check_draft.repeat_kind == kind, kind.label())
                            .clicked()
                        {
                            self.new_check_draft.repeat_kind = kind;
                            if kind == RepeatKind::Everytime {
                                self.new_check_draft.repeat_value.clear();
                            } else if self.new_check_draft.repeat_value.trim().is_empty() {
                                self.new_check_draft.repeat_value = "1".to_string();
                            }
                        }
                    }
                });

            if self.new_check_draft.repeat_kind != RepeatKind::Everytime {
                labeled_text_edit(
                    ui,
                    theme,
                    "Repeat value",
                    &mut self.new_check_draft.repeat_value,
                    false,
                );
            } else {
                ui.add_space(theme.spacing_md);
            }

            labeled_text_edit(ui, theme, "Position", &mut self.new_check_draft.position, false);

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.new_check_draft.is_mandatory, "");
                ui.label(RichText::new("Mandatory").color(theme.text_secondary));
            });
            ui.add_space(theme.spacing_sm);

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.new_check_draft.is_checked, "");
                ui.label(RichText::new("Initially checked").color(theme.text_secondary));
            });
            ui.add_space(theme.spacing_lg);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    self.new_check_draft = NewCheckDraft::default();
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }

                let save_enabled = !self.new_check_draft.name.trim().is_empty();
                if ui
                    .add_enabled(save_enabled, egui::Button::new("Save"))
                    .clicked()
                {
                    match self.insert_new_check() {
                        Ok(()) => {
                            self.new_check_draft = NewCheckDraft::default();
                            self.mode = ContentMode::General;
                            self.error_message = None;
                        }
                        Err(error) => self.error_message = Some(error),
                    }
                }
            });
        });
    }

    fn show_comments_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        egui::Frame::new()
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| {
                ui.heading(RichText::new("Comments").color(theme.text_primary));
                ui.label(
                    RichText::new("Comments content is planned for a future iteration.")
                        .color(theme.text_secondary),
                );
            });
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
            repeat_kind: RepeatKind::Everytime,
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

        let repeat_case = match self.repeat_kind {
            RepeatKind::Everytime => CheckRepeatType::Everytime,
            RepeatKind::Conditional => {
                CheckRepeatType::Conditional(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
            RepeatKind::Specific => {
                CheckRepeatType::Specific(parse_positive_i32(&self.repeat_value, "Repeat value")?)
            }
            RepeatKind::Until => {
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

impl RepeatKind {
    fn label(&self) -> &'static str {
        match self {
            RepeatKind::Everytime => "Everytime",
            RepeatKind::Conditional => "Conditional",
            RepeatKind::Specific => "Specific",
            RepeatKind::Until => "Until",
        }
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

fn labeled_text_edit(
    ui: &mut egui::Ui,
    theme: &Theme,
    label: &str,
    value: &mut String,
    multiline: bool,
) {
    ui.label(RichText::new(label).color(theme.text_secondary));
    if multiline {
        ui.add(
            egui::TextEdit::multiline(value)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
    } else {
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
    }
    ui.add_space(theme.spacing_md);
}

fn field_row(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &str) {
    ui.label(RichText::new(label).color(theme.text_muted));
    ui.label(RichText::new(value).color(theme.text_secondary));
    ui.end_row();
}

fn source_label(source: &CheckSourceType) -> &'static str {
    match source {
        CheckSourceType::Game => "Game",
        CheckSourceType::GlobalGame => "Global Game",
        CheckSourceType::Blueprint => "Blueprint",
        CheckSourceType::Turn => "Turn",
    }
}

fn repeat_label(repeat_case: &CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => "Every turn".to_string(),
        CheckRepeatType::Conditional(value) => format!("Conditional ({value})"),
        CheckRepeatType::Specific(value) => format!("Specific ({value})"),
        CheckRepeatType::Until(value) => format!("Until ({value})"),
    }
}

fn source_color(check: &Check, theme: &Theme) -> egui::Color32 {
    match check.source {
        CheckSourceType::Blueprint => theme.source_blueprint,
        CheckSourceType::Game => theme.source_game,
        CheckSourceType::GlobalGame => theme.source_global,
        CheckSourceType::Turn => theme.source_turn,
    }
}

fn repeat_color(repeat_case: &CheckRepeatType, theme: &Theme) -> egui::Color32 {
    match repeat_case {
        CheckRepeatType::Everytime => theme.repeat_everytime,
        CheckRepeatType::Conditional(_) => theme.repeat_conditional,
        CheckRepeatType::Specific(_) => theme.repeat_specific,
        CheckRepeatType::Until(_) => theme.repeat_until,
    }
}

fn status_badge(ui: &mut egui::Ui, theme: &Theme, repeat_case: &CheckRepeatType) {
    egui::Frame::new()
        .fill(repeat_color(repeat_case, theme))
        .corner_radius(theme.corner_radius)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(repeat_label(repeat_case)).color(theme.text_primary).small());
        });
}

#[cfg(test)]
mod tests {
    use super::{NewCheckDraft, RepeatKind};
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
            repeat_kind: RepeatKind::Until,
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
            repeat_kind: RepeatKind::Specific,
            repeat_value: "4".to_string(),
            ..Default::default()
        };

        let check = draft.to_check().expect("draft should convert");
        assert_eq!(check.repeat_case, CheckRepeatType::Specific(4));
    }
}
