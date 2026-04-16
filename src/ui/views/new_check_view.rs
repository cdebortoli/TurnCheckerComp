use eframe::egui::{self, RichText};

use super::new_check_draft::{NewCheckDraft, MAX_REPEAT_VALUE};
use crate::i18n::I18n;
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, CurrentSession, Tag};
use crate::ui::components::toggle_button::toggle;
use crate::ui::theme::Theme;
use crate::ui::ui_helpers::{find_tag_by_uuid, tag_fill_color};

#[derive(Default)]
pub(super) struct NewCheckView {
    draft: NewCheckDraft,
}

pub(super) enum NewCheckAction {
    Cancelled,
    SaveNewRequested(Check),
    SaveExistingRequested(Check),
    ValidationFailed(String),
}

impl NewCheckView {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        tags: &[Tag],
        current_session: Option<&CurrentSession>,
    ) -> Option<NewCheckAction> {
        self.show_new_check_heading(ui, theme, i18n);

        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_new_check_text_fields(ui, theme, i18n);
            ui.horizontal(|ui| {
                self.show_new_check_tag_selector(ui, theme, i18n, tags);
                self.show_new_check_source_selector(ui, theme, i18n, current_session);
                ui.vertical(|ui| {
                    self.show_new_check_repeat_selector(ui, theme, i18n);
                    self.show_new_check_repeat_value(ui, theme, i18n);
                });
            });
            self.show_new_check_toggles(ui, theme, i18n);
            action = self.show_new_check_actions(ui, i18n, current_session);
        });

        action
    }

    pub(super) fn prepare_new(&mut self) {
        self.reset_new_check_form();
    }

    pub(super) fn start_editing(&mut self, check: &Check) {
        self.draft = NewCheckDraft::from_check(check);
    }

    pub(super) fn reset(&mut self) {
        self.reset_new_check_form();
    }

    fn show_new_check_heading(&self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        ui.heading(RichText::new(i18n.t("new-check-title")).color(theme.text_primary));
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_text_fields(&mut self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        labeled_text_edit(
            ui,
            theme,
            &i18n.t("field-name"),
            &mut self.draft.name,
            false,
            None,
        );
        labeled_text_edit(
            ui,
            theme,
            &i18n.t("field-detail"),
            &mut self.draft.detail,
            true,
            None,
        );
    }

    fn show_new_check_source_selector(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        current_session: Option<&CurrentSession>,
    ) {
        ui.label(RichText::new(i18n.t("field-source")).color(theme.text_secondary));
        ui.add_enabled_ui(!self.draft.source_is_locked(), |ui| {
            egui::ComboBox::from_id_salt("new_check_source")
                .selected_text(source_label(i18n, &self.draft.source))
                .show_ui(ui, |ui| {
                    for source in [
                        CheckSourceType::Game,
                        CheckSourceType::GlobalGame,
                        CheckSourceType::Blueprint,
                        CheckSourceType::Turn,
                    ] {
                        let label = source_label(i18n, &source);
                        let is_selected = self.draft.source == source;
                        if show_colored_option(
                            ui,
                            &label,
                            source_color(&source, theme),
                            is_selected,
                        )
                        .clicked()
                        {
                            self.draft.set_source(source, current_session);
                            ui.close();
                        }
                    }
                });
        });
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_tag_selector(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        tags: &[Tag],
    ) {
        ui.label(RichText::new(i18n.t("field-tag")).color(theme.text_secondary));

        let selected_label = find_tag_by_uuid(tags, self.draft.selected_tag_uuid)
            .map(|tag| tag.name.clone())
            .unwrap_or_else(|| i18n.t("field-no-tag"));

        egui::ComboBox::from_id_salt("new_check_tag")
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.draft.selected_tag_uuid,
                    None,
                    i18n.t("field-no-tag"),
                );

                for tag in tags {
                    let is_selected = self.draft.selected_tag_uuid == Some(tag.uuid);
                    if show_tag_option(ui, theme, tag, is_selected).clicked() {
                        self.draft.selected_tag_uuid = Some(tag.uuid);
                        ui.close();
                    }
                }
            });
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_repeat_selector(&mut self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        let repeat_locked = self.draft.turn_repeat_is_locked();
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("field-repeat")).color(theme.text_secondary));
            ui.add_enabled_ui(!repeat_locked, |ui| {
                egui::ComboBox::from_id_salt("new_check_repeat_kind")
                    .selected_text(repeat_label(i18n, &self.draft.repeat_case))
                    .show_ui(ui, |ui| {
                        for repeat_case in [
                            CheckRepeatType::Everytime,
                            CheckRepeatType::Conditional(1),
                            CheckRepeatType::Specific(1),
                            CheckRepeatType::Until(1),
                        ] {
                            let label = repeat_label(i18n, &repeat_case);
                            if show_colored_option(
                                ui,
                                &label,
                                repeat_color(&repeat_case, theme),
                                same_repeat_variant(&self.draft.repeat_case, &repeat_case),
                            )
                            .clicked()
                            {
                                self.draft.repeat_case = repeat_case;
                                self.normalize_repeat_value();
                                ui.close();
                            }
                        }
                    });
            });
        });
    }

    fn show_new_check_repeat_value(&mut self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        if repeat_requires_value(&self.draft.repeat_case) {
            let repeat_locked = self.draft.turn_repeat_is_locked();
            ui.horizontal(|ui| {
                ui.add_enabled_ui(!repeat_locked, |ui| {
                    labeled_numeric_text_edit(
                        ui,
                        theme,
                        &i18n.t("field-repeat-value"),
                        &mut self.draft.repeat_value,
                        Some(40.0),
                        MAX_REPEAT_VALUE,
                    );
                });
            });
        } else {
            ui.add_space(theme.spacing_md);
        }
    }

    fn show_new_check_toggles(&mut self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("field-mandatory")).color(theme.text_secondary));
            ui.add(toggle(&mut self.draft.is_mandatory, theme));
        });
        ui.add_space(theme.spacing_lg);
    }

    fn show_new_check_actions(
        &mut self,
        ui: &mut egui::Ui,
        i18n: &I18n,
        current_session: Option<&CurrentSession>,
    ) -> Option<NewCheckAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            if ui.button(i18n.t("action-cancel")).clicked() {
                self.reset_new_check_form();
                action = Some(NewCheckAction::Cancelled);
            }

            let save_enabled = !self.draft.name.trim().is_empty();
            if ui
                .add_enabled(save_enabled, egui::Button::new(i18n.t("action-save")))
                .clicked()
            {
                action = Some(match self.draft.to_check(i18n, current_session) {
                    Ok(check) if self.draft.is_editing() => {
                        NewCheckAction::SaveExistingRequested(check)
                    }
                    Ok(check) => NewCheckAction::SaveNewRequested(check),
                    Err(error) => NewCheckAction::ValidationFailed(error),
                });
            }
        });

        action
    }

    fn normalize_repeat_value(&mut self) {
        if !repeat_requires_value(&self.draft.repeat_case) {
            self.draft.repeat_value.clear();
        } else if self.draft.repeat_value.trim().is_empty() {
            self.draft.repeat_value = "1".to_string();
        }
    }

    fn reset_new_check_form(&mut self) {
        self.draft = Default::default();
    }
}

fn labeled_text_edit(
    ui: &mut egui::Ui,
    theme: &Theme,
    label: &str,
    value: &mut String,
    multiline: bool,
    width: Option<f32>,
) {
    ui.label(RichText::new(label).color(theme.text_secondary));
    if multiline {
        ui.add(
            egui::TextEdit::multiline(value)
                .desired_rows(3)
                .background_color(theme.bg_list_element)
                .desired_width(width.unwrap_or(f32::INFINITY)),
        );
    } else {
        ui.add(
            egui::TextEdit::singleline(value)
                .background_color(theme.bg_list_element)
                .desired_width(width.unwrap_or(f32::INFINITY)),
        );
    }
    ui.add_space(theme.spacing_md);
}

fn labeled_numeric_text_edit(
    ui: &mut egui::Ui,
    theme: &Theme,
    label: &str,
    value: &mut String,
    width: Option<f32>,
    max_value: i32,
) {
    ui.label(RichText::new(label).color(theme.text_secondary));
    ui.add(
        egui::TextEdit::singleline(value)
            .background_color(theme.bg_list_element)
            .desired_width(width.unwrap_or(f32::INFINITY)),
    );
    sanitize_numeric_input(value, max_value);
    ui.add_space(theme.spacing_md);
}

fn sanitize_numeric_input(value: &mut String, max_value: i32) {
    value.retain(|ch| ch.is_ascii_digit());

    if value.is_empty() {
        return;
    }

    match value.parse::<i32>() {
        Ok(parsed) if parsed > max_value => *value = max_value.to_string(),
        Ok(_) => {}
        Err(_) => *value = max_value.to_string(),
    }
}

fn source_label(i18n: &I18n, source: &CheckSourceType) -> String {
    match source {
        CheckSourceType::Game => i18n.t("source-game"),
        CheckSourceType::GlobalGame => i18n.t("source-global-game"),
        CheckSourceType::Blueprint => i18n.t("source-blueprint"),
        CheckSourceType::Turn => i18n.t("source-turn"),
    }
}

fn repeat_label(i18n: &I18n, repeat_case: &CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => i18n.t("repeat-everytime"),
        CheckRepeatType::Conditional(_) => i18n.t("repeat-conditional"),
        CheckRepeatType::Specific(_) => i18n.t("repeat-specific"),
        CheckRepeatType::Until(_) => i18n.t("repeat-until"),
    }
}

fn repeat_requires_value(repeat_case: &CheckRepeatType) -> bool {
    !matches!(repeat_case, CheckRepeatType::Everytime)
}

fn show_tag_option(
    ui: &mut egui::Ui,
    theme: &Theme,
    tag: &Tag,
    is_selected: bool,
) -> egui::Response {
    show_colored_option(ui, &tag.name, tag_fill_color(tag, theme), is_selected)
}

fn show_colored_option(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    is_selected: bool,
) -> egui::Response {
    let response = ui.add(
        egui::Button::new(
            egui::RichText::new(format!("   {label}")).color(ui.visuals().text_color()),
        )
        .min_size(egui::vec2(ui.available_width(), 0.0))
        .selected(is_selected),
    );

    ui.painter().circle_filled(
        egui::pos2(response.rect.left() + 6.0, response.rect.center().y),
        4.0,
        color,
    );

    response
}

fn source_color(source: &CheckSourceType, theme: &Theme) -> egui::Color32 {
    match source {
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

fn same_repeat_variant(left: &CheckRepeatType, right: &CheckRepeatType) -> bool {
    matches!(
        (left, right),
        (CheckRepeatType::Everytime, CheckRepeatType::Everytime)
            | (
                CheckRepeatType::Conditional(_),
                CheckRepeatType::Conditional(_)
            )
            | (CheckRepeatType::Specific(_), CheckRepeatType::Specific(_))
            | (CheckRepeatType::Until(_), CheckRepeatType::Until(_))
    )
}
