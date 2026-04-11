use eframe::egui::{self, RichText};

use super::new_check_draft::NewCheckDraft;
use super::helpers::{find_tag_by_uuid, tag_fill_color};
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, Tag};
use crate::ui::content::toggle_button::toggle;
use crate::ui::theme::Theme;

#[derive(Default)]
pub(super) struct NewCheckView {
    draft: NewCheckDraft,
}

pub(super) enum NewCheckAction {
    Cancelled,
    SaveRequested(Check),
    ValidationFailed(String),
}

impl NewCheckView {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        tags: &[Tag],
    ) -> Option<NewCheckAction> {
        self.show_new_check_heading(ui, theme);

        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_new_check_text_fields(ui, theme);
            ui.horizontal(|ui| {
                self.show_new_check_tag_selector(ui, theme, tags);
                self.show_new_check_source_selector(ui, theme);
                ui.vertical(|ui| {
                    self.show_new_check_repeat_selector(ui, theme);
                    self.show_new_check_repeat_value(ui, theme);
                });
            });
            self.show_new_check_toggles(ui, theme);
            action = self.show_new_check_actions(ui);
        });

        action
    }

    pub(super) fn reset(&mut self) {
        self.reset_new_check_form();
    }

    fn show_new_check_heading(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading(RichText::new("Create a new check").color(theme.text_primary));
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_text_fields(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        labeled_text_edit(ui, theme, "Name", &mut self.draft.name, false, None);
        labeled_text_edit(ui, theme, "Detail", &mut self.draft.detail, true, None);
    }

    fn show_new_check_source_selector(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("Source").color(theme.text_secondary));
        egui::ComboBox::from_id_salt("new_check_source")
            .selected_text(source_label(&self.draft.source))
            .show_ui(ui, |ui| {
                for source in [
                    CheckSourceType::Game,
                    CheckSourceType::GlobalGame,
                    CheckSourceType::Blueprint,
                    CheckSourceType::Turn,
                ] {
                    let is_selected = self.draft.source == source;
                    if show_colored_option(
                        ui,
                        source_label(&source),
                        source_color(&source, theme),
                        is_selected,
                    )
                    .clicked()
                    {
                        self.draft.source = source;
                        ui.close();
                    }
                }
            });
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_tag_selector(&mut self, ui: &mut egui::Ui, theme: &Theme, tags: &[Tag]) {
        ui.label(RichText::new("Tag").color(theme.text_secondary));

        let selected_label = find_tag_by_uuid(tags, self.draft.selected_tag_uuid)
            .map(|tag| tag.name.as_str())
            .unwrap_or("No tag");

        egui::ComboBox::from_id_salt("new_check_tag")
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.draft.selected_tag_uuid, None, "No tag");

                for tag in tags {
                    let is_selected = self.draft.selected_tag_uuid == Some(tag.uuid);
                    if show_tag_option(ui, tag, is_selected).clicked() {
                        self.draft.selected_tag_uuid = Some(tag.uuid);
                        ui.close();
                    }
                }
            });
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_repeat_selector(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Repeat").color(theme.text_secondary));
            egui::ComboBox::from_id_salt("new_check_repeat_kind")
                .selected_text(repeat_label(&self.draft.repeat_case))
                .show_ui(ui, |ui| {
                    for repeat_case in [
                        CheckRepeatType::Everytime,
                        CheckRepeatType::Conditional(1),
                        CheckRepeatType::Specific(1),
                        CheckRepeatType::Until(1),
                    ] {
                        if show_colored_option(
                            ui,
                            repeat_label(&repeat_case),
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
    }

    fn show_new_check_repeat_value(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        if repeat_requires_value(&self.draft.repeat_case) {
            ui.horizontal(|ui| {
                labeled_text_edit(
                    ui,
                    theme,
                    "Repeat value",
                    &mut self.draft.repeat_value,
                    false,
                    Some(40.0),
                );
            });
        } else {
            ui.add_space(theme.spacing_md);
        }
    }

    fn show_new_check_toggles(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Mandatory").color(theme.text_secondary));
            ui.add(toggle(&mut self.draft.is_mandatory, theme));
        });
        ui.add_space(theme.spacing_lg);
    }

    fn show_new_check_actions(&mut self, ui: &mut egui::Ui) -> Option<NewCheckAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.reset_new_check_form();
                action = Some(NewCheckAction::Cancelled);
            }

            let save_enabled = !self.draft.name.trim().is_empty();
            if ui
                .add_enabled(save_enabled, egui::Button::new("Save"))
                .clicked()
            {
                action = Some(match self.draft.to_check() {
                    Ok(check) => NewCheckAction::SaveRequested(check),
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

fn source_label(source: &CheckSourceType) -> &'static str {
    match source {
        CheckSourceType::Game => "Game",
        CheckSourceType::GlobalGame => "Global Game",
        CheckSourceType::Blueprint => "Blueprint",
        CheckSourceType::Turn => "Turn",
    }
}

fn repeat_label(repeat_case: &CheckRepeatType) -> &'static str {
    match repeat_case {
        CheckRepeatType::Everytime => "Everytime",
        CheckRepeatType::Conditional(_) => "Conditional",
        CheckRepeatType::Specific(_) => "Specific",
        CheckRepeatType::Until(_) => "Until",
    }
}

fn repeat_requires_value(repeat_case: &CheckRepeatType) -> bool {
    !matches!(repeat_case, CheckRepeatType::Everytime)
}

fn show_tag_option(ui: &mut egui::Ui, tag: &Tag, is_selected: bool) -> egui::Response {
    show_colored_option(ui, &tag.name, tag_fill_color(tag), is_selected)
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
