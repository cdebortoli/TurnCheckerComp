use eframe::egui::{self, RichText};

use super::{ContentMode, MainContentView};
use crate::models::check_source_type::CheckSourceType;
use crate::models::CheckRepeatType;
use crate::ui::theme::Theme;

impl MainContentView {
    pub(super) fn show_new_check_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        self.show_new_check_heading(ui, theme);

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_new_check_text_fields(ui, theme);
            self.show_new_check_source_selector(ui, theme);
            self.show_new_check_repeat_selector(ui, theme);
            self.show_new_check_repeat_value(ui, theme);
            self.show_new_check_position(ui, theme);
            self.show_new_check_toggles(ui, theme);
            self.show_new_check_actions(ui);
        });
    }

    fn show_new_check_heading(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading(RichText::new("Create a new check").color(theme.text_primary));
        ui.label(RichText::new("Set all parameters before saving.").color(theme.text_secondary));
        ui.add_space(theme.spacing_md);
    }

    fn show_new_check_text_fields(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        labeled_text_edit(ui, theme, "Name", &mut self.new_check_draft.name, false);
        labeled_text_edit(ui, theme, "Detail", &mut self.new_check_draft.detail, true);
    }

    fn show_new_check_source_selector(&mut self, ui: &mut egui::Ui, theme: &Theme) {
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
    }

    fn show_new_check_repeat_selector(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("Repeat").color(theme.text_secondary));
        egui::ComboBox::from_id_salt("new_check_repeat_kind")
            .selected_text(repeat_label(&self.new_check_draft.repeat_case))
            .show_ui(ui, |ui| {
                for repeat_case in [
                    CheckRepeatType::Everytime,
                    CheckRepeatType::Conditional(1),
                    CheckRepeatType::Specific(1),
                    CheckRepeatType::Until(1),
                ] {
                    if ui
                        .selectable_label(
                            same_repeat_variant(&self.new_check_draft.repeat_case, &repeat_case),
                            repeat_label(&repeat_case),
                        )
                        .clicked()
                    {
                        self.new_check_draft.repeat_case = repeat_case;
                        self.normalize_repeat_value();
                    }
                }
            });
    }

    fn show_new_check_repeat_value(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        if repeat_requires_value(&self.new_check_draft.repeat_case) {
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
    }

    fn show_new_check_position(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        labeled_text_edit(ui, theme, "Position", &mut self.new_check_draft.position, false);
    }

    fn show_new_check_toggles(&mut self, ui: &mut egui::Ui, theme: &Theme) {
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
    }

    fn show_new_check_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.reset_new_check_form();
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
                        self.reset_new_check_form();
                        self.mode = ContentMode::General;
                        self.error_message = None;
                    }
                    Err(error) => self.error_message = Some(error),
                }
            }
        });
    }

    fn normalize_repeat_value(&mut self) {
        if !repeat_requires_value(&self.new_check_draft.repeat_case) {
            self.new_check_draft.repeat_value.clear();
        } else if self.new_check_draft.repeat_value.trim().is_empty() {
            self.new_check_draft.repeat_value = "1".to_string();
        }
    }

    fn reset_new_check_form(&mut self) {
        self.new_check_draft = Default::default();
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

fn same_repeat_variant(left: &CheckRepeatType, right: &CheckRepeatType) -> bool {
    matches!(
        (left, right),
        (CheckRepeatType::Everytime, CheckRepeatType::Everytime)
            | (CheckRepeatType::Conditional(_), CheckRepeatType::Conditional(_))
            | (CheckRepeatType::Specific(_), CheckRepeatType::Specific(_))
            | (CheckRepeatType::Until(_), CheckRepeatType::Until(_))
    )
}
