use eframe::egui::{self, RichText};

use super::MainContentView;
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType};
use crate::ui::theme::Theme;

impl MainContentView {
    pub(super) fn show_general_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        self.show_checklist_header(ui, theme);

        if self.checks.is_empty() {
            self.show_empty_checklist(ui, theme);
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for check in self.checks.clone() {
                    self.show_check_card(ui, theme, check);
                    ui.add_space(theme.spacing_md);
                }
            });
    }

    fn show_checklist_header(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("All checks").color(theme.text_secondary));
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_checklist(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("No checks yet.").color(theme.text_muted));
    }

    fn show_check_card(&mut self, ui: &mut egui::Ui, theme: &Theme, check: Check) {
        let mut selected_checked = check.is_checked;

        egui::Frame::new()
            .fill(theme.bg_list_element)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| {
                self.show_check_card_header(ui, theme, &check, &mut selected_checked);
                self.show_check_card_details(ui, theme, &check);
            });

        if selected_checked != check.is_checked {
            if let Err(error) = self.update_check_status(check, selected_checked) {
                self.error_message = Some(error);
            }
        }
    }

    fn show_check_card_header(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        check: &Check,
        selected_checked: &mut bool,
    ) {
        ui.horizontal(|ui| {
            self.show_check_source_indicator(ui, theme, check);
            self.show_check_card_title(ui, theme, check);
            self.show_check_status_selector(ui, check, selected_checked);
        });
    }

    fn show_check_source_indicator(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 36.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, theme.corner_radius, source_color(check, theme));
    }

    fn show_check_card_title(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&check.name)
                        .color(theme.text_primary)
                        .strong(),
                );
                if check.is_mandatory {
                    ui.label(RichText::new("Mandatory").color(theme.warning).small());
                }
                show_repeat_badge(ui, theme, &check.repeat_case);
            });

            if let Some(detail) = &check.detail {
                if !detail.is_empty() {
                    ui.label(RichText::new(detail).color(theme.text_secondary));
                }
            }
        });
    }

    fn show_check_status_selector(
        &self,
        ui: &mut egui::Ui,
        check: &Check,
        selected_checked: &mut bool,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            egui::ComboBox::from_id_salt(("check_status", check.id))
                .selected_text(if *selected_checked {
                    "Checked"
                } else {
                    "Unchecked"
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(selected_checked, false, "Unchecked");
                    ui.selectable_value(selected_checked, true, "Checked");
                });
        });
    }

    fn show_check_card_details(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        ui.add_space(theme.spacing_sm);
        ui.separator();
        ui.add_space(theme.spacing_sm);

        let details = CheckCardDetails::from(check);
        egui::Grid::new(("check_grid", check.id))
            .num_columns(2)
            .spacing([theme.spacing_lg, theme.spacing_sm])
            .show(ui, |ui| {
                field_row(ui, theme, "Source", source_label(&check.source));
                field_row(ui, theme, "Repeat", &details.repeat_text);
                field_row(ui, theme, "Position", &details.position_text);
                field_row(ui, theme, "Selected", details.selected_text);
                field_row(ui, theme, "Mandatory", details.mandatory_text);
                field_row(ui, theme, "Sent", details.sent_text);
                field_row(ui, theme, "UUID", &details.uuid_text);
            });
    }
}

struct CheckCardDetails {
    repeat_text: String,
    position_text: String,
    selected_text: &'static str,
    mandatory_text: &'static str,
    sent_text: &'static str,
    uuid_text: String,
}

impl From<&Check> for CheckCardDetails {
    fn from(check: &Check) -> Self {
        Self {
            repeat_text: repeat_label(&check.repeat_case),
            position_text: check.position.to_string(),
            selected_text: if check.is_checked {
                "Checked"
            } else {
                "Unchecked"
            },
            mandatory_text: if check.is_mandatory { "Yes" } else { "No" },
            sent_text: if check.is_sent { "Yes" } else { "No" },
            uuid_text: check.uuid.to_string(),
        }
    }
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

fn show_repeat_badge(ui: &mut egui::Ui, theme: &Theme, repeat_case: &CheckRepeatType) {
    egui::Frame::new()
        .fill(repeat_color(repeat_case, theme))
        .corner_radius(theme.corner_radius)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(
                RichText::new(repeat_label(repeat_case))
                    .color(theme.text_primary)
                    .small(),
            );
        });
}
