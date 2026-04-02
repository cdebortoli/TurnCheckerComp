use eframe::egui::{self, RichText};
use egui::Color32;

use super::{find_tag_by_uuid, show_tag_capsule, MainContentView};
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType};
use crate::ui::content::toggle_button::toggle;
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
        let row = ui.horizontal(|ui| {
            let (indicator_rect, _) =
                ui.allocate_exact_size(egui::vec2(4.0, 1.0), egui::Sense::hover());
            self.show_check_card_title(ui, theme, check);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.show_sent_status_icon(ui, theme, check);
                self.show_check_toggle(ui, selected_checked, theme);
                self.show_mandatory_indicator(ui, theme, check);
            });
            indicator_rect
        });

        self.show_check_source_indicator(ui, theme, check, row.inner, row.response.rect);
    }

    fn show_check_source_indicator(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        check: &Check,
        indicator_rect: egui::Rect,
        row_rect: egui::Rect,
    ) {
        let inset = theme.spacing_sm;
        let top = (row_rect.top() + inset).min(row_rect.bottom());
        let bottom = (row_rect.bottom() - inset).max(top);
        let rect = egui::Rect::from_min_max(
            egui::pos2(indicator_rect.left(), top),
            egui::pos2(indicator_rect.right(), bottom),
        );

        ui.painter()
            .rect_filled(rect, theme.corner_radius, source_color(check, theme));
    }

    fn show_check_card_title(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        ui.vertical(|ui| {
            // First line
            ui.horizontal_wrapped(|ui| {
                show_repeat_badge(ui, theme, &check.repeat_case);

                if let Some(tag) = find_tag_by_uuid(&self.tags, check.tag_uuid) {
                    show_tag_capsule(ui, tag);
                }
            });

            ui.add_space(theme.spacing_sm);

            // Second line
            ui.label(
                RichText::new(&check.name)
                    .color(theme.text_primary)
                    .family(egui::FontFamily::Name("montserrat-bold".into())),
            );

            // Third line
            if let Some(detail) = &check.detail {
                ui.add_space(theme.spacing_sm);
                ui.label(RichText::new(detail).color(theme.text_secondary).small());
            }
        });
    }

    fn show_mandatory_indicator(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        if check.is_mandatory {
            ui.label(RichText::new("Mandatory").color(theme.warning).small());
        }
    }

    fn show_check_toggle(&mut self, ui: &mut egui::Ui, selected_checked: &mut bool, theme: &Theme) {
        ui.add(toggle(selected_checked, theme));
    }

    fn show_sent_status_icon(&self, ui: &mut egui::Ui, theme: &Theme, check: &Check) {
        let circle_color = if check.is_sent {
            eframe::egui::Color32::from_rgba_premultiplied(48, 209, 88, 220)
        } else {
            eframe::egui::Color32::from_rgba_premultiplied(255, 69, 58, 220)
        };
        let icon_color = theme.text_primary;
        let size = 20.0;
        let stroke = egui::Stroke::new(2.0, icon_color);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        let center = rect.center();
        let radius = size * 0.5;

        ui.painter().circle_filled(center, radius, circle_color);

        if check.is_sent {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 5.5, rect.center().y + 1.5),
                    egui::pos2(rect.left() + 9.5, rect.bottom() - 6.0),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 9.5, rect.bottom() - 6.0),
                    egui::pos2(rect.right() - 5.0, rect.top() + 6.0),
                ],
                stroke,
            );
        } else {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                    egui::pos2(rect.right() - 6.0, rect.bottom() - 6.0),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 6.0, rect.bottom() - 6.0),
                    egui::pos2(rect.right() - 6.0, rect.top() + 6.0),
                ],
                stroke,
            );
        }
    }
}

fn repeat_label(repeat_case: &CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => "Every turn".to_string(),
        CheckRepeatType::Conditional(value) => format!("Conditional (Turn {value})"),
        CheckRepeatType::Specific(value) => format!("Specific (Turn {value})"),
        CheckRepeatType::Until(value) => format!("Until (Turn {value})"),
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
                    .color(Color32::WHITE)
                    .family(egui::FontFamily::Name("montserrat-bold".into()))
                    .small(),
            );
        });
}
