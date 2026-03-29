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
            });

        if selected_checked != check.is_checked {
            if let Err(error) = self.update_check_status(check, selected_checked) {
                self.error_message = Some(error);
            }
        }
    }

    fn show_check_card_header(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        check: &Check,
        selected_checked: &mut bool,
    ) {
        ui.horizontal(|ui| {
            self.show_check_source_indicator(ui, theme, check);
            self.show_check_card_title(ui, theme, check);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.show_sent_status_icon(ui, theme, check);
                self.show_check_toggle(ui, check, selected_checked);
                self.show_mandatory_indicator(ui, theme, check);
            });
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
                show_repeat_badge(ui, theme, &check.repeat_case);
            });
        });
    }

    fn show_mandatory_indicator(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        if check.is_mandatory {
            ui.label(RichText::new("Mandatory").color(theme.warning).small());
        }
    }

    fn show_check_toggle(&mut self, ui: &mut egui::Ui, check: &Check, selected_checked: &mut bool) {
        if ui.checkbox(selected_checked, "").changed() {
            let is_checked = *selected_checked;
            if let Err(error) = self.update_check_status(check.clone(), is_checked) {
                self.error_message = Some(error);
            }
        }
    }

    fn show_sent_status_icon(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        let color = if check.is_sent {
            theme.success
        } else {
            theme.destructive
        };
        let circle_color = if check.is_sent {
            eframe::egui::Color32::from_rgba_premultiplied(48, 209, 88, 200)
        } else {
            eframe::egui::Color32::from_rgba_premultiplied(255, 69, 58, 200)
        };

        egui::Frame::new()
            .fill(circle_color)
            .corner_radius(theme.corner_radius)
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |ui| {
                ui.painter()
                    .circle_filled(eframe::egui::pos2(6.0, 6.0), 6.0, color);
            });
    }
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
