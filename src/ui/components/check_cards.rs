use eframe::egui::{self, RichText};
use egui::Color32;

use crate::i18n::{I18n, I18nValue};
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType, Tag};
use crate::ui::components::toggle_button::toggle;
use crate::ui::theme::Theme;
use crate::ui::ui_helpers::{find_tag_by_uuid, show_sent_status_icon, show_tag_capsule};

#[derive(Default)]
pub(crate) struct CheckCardsView;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckCardDisplayMode {
    Toggleable,
    ReadOnly,
}

pub(crate) enum CheckCardsAction {
    CheckToggled { check: Check, is_checked: bool },
    CheckSelected(Check),
}

impl CheckCardsView {
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        checks: &[Check],
        tags: &[Tag],
        display_mode: CheckCardDisplayMode,
    ) -> Option<CheckCardsAction> {
        let mut action = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for check in checks.iter().cloned() {
                    // While no previous action different to none, updated it.
                    // When a card_action is different of none, it means that the action will be managed, then after the redraw, it will reset to none
                    // So next new action will be able to be managed
                    let card_action =
                        self.show_check_card(ui, theme, i18n, tags, check, display_mode);
                    if action.is_none() {
                        action = card_action;
                    }
                    ui.add_space(theme.spacing_md);
                }
            });

        action
    }

    fn show_check_card(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        tags: &[Tag],
        check: Check,
        display_mode: CheckCardDisplayMode,
    ) -> Option<CheckCardsAction> {
        let mut selected_checked = check.is_checked;

        let card = egui::Frame::new()
            .fill(theme.bg_list_element)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| {
                self.show_check_card_header(
                    ui,
                    theme,
                    i18n,
                    tags,
                    &check,
                    &mut selected_checked,
                    display_mode,
                );
            });

        let card_response = ui.interact(
            card.response.rect,
            ui.make_persistent_id(("check_card", check.uuid)),
            egui::Sense::click(),
        );

        if display_mode == CheckCardDisplayMode::Toggleable && selected_checked != check.is_checked
        {
            Some(CheckCardsAction::CheckToggled {
                check,
                is_checked: selected_checked,
            })
        } else if card_response.clicked() {
            Some(CheckCardsAction::CheckSelected(check))
        } else {
            None
        }
    }

    fn show_check_card_header(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        tags: &[Tag],
        check: &Check,
        selected_checked: &mut bool,
        display_mode: CheckCardDisplayMode,
    ) {
        let row = ui.horizontal(|ui| {
            let (indicator_rect, _) =
                ui.allocate_exact_size(egui::vec2(4.0, 1.0), egui::Sense::hover());
            self.show_check_card_title(ui, theme, i18n, tags, check);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                show_sent_status_icon(ui, theme, check.is_sent);
                self.show_check_toggle(ui, selected_checked, theme, display_mode);
                self.show_mandatory_indicator(ui, theme, i18n, check);
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

    fn show_check_card_title(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        tags: &[Tag],
        check: &Check,
    ) {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                show_repeat_badge(ui, theme, i18n, &check.repeat_case);

                if let Some(tag) = find_tag_by_uuid(tags, check.tag_uuid) {
                    show_tag_capsule(ui, tag);
                }
            });

            ui.add_space(theme.spacing_sm);

            ui.label(
                RichText::new(&check.name)
                    .color(theme.text_primary)
                    .family(egui::FontFamily::Name("montserrat-bold".into())),
            );

            if let Some(detail) = &check.detail {
                ui.add_space(theme.spacing_sm);
                ui.label(RichText::new(detail).color(theme.text_secondary).small());
            }
        });
    }

    fn show_mandatory_indicator(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        check: &Check,
    ) {
        if check.is_mandatory {
            ui.label(
                RichText::new(i18n.t("check-mandatory"))
                    .color(theme.warning)
                    .small(),
            );
        }
    }

    fn show_check_toggle(
        &mut self,
        ui: &mut egui::Ui,
        selected_checked: &mut bool,
        theme: &Theme,
        display_mode: CheckCardDisplayMode,
    ) {
        match display_mode {
            CheckCardDisplayMode::Toggleable => {
                ui.add(toggle(selected_checked, theme));
            }
            CheckCardDisplayMode::ReadOnly => {}
        }
    }
}

fn repeat_label(i18n: &I18n, repeat_case: &CheckRepeatType) -> String {
    match repeat_case {
        CheckRepeatType::Everytime => i18n.t("repeat-badge-every-turn"),
        CheckRepeatType::Conditional(value) => i18n.tr(
            "repeat-badge-conditional",
            &[("turn", I18nValue::from(*value))],
        ),
        CheckRepeatType::Specific(value) => i18n.tr(
            "repeat-badge-specific",
            &[("turn", I18nValue::from(*value))],
        ),
        CheckRepeatType::Until(value) => {
            i18n.tr("repeat-badge-until", &[("turn", I18nValue::from(*value))])
        }
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

fn show_repeat_badge(ui: &mut egui::Ui, theme: &Theme, i18n: &I18n, repeat_case: &CheckRepeatType) {
    egui::Frame::new()
        .fill(repeat_color(repeat_case, theme))
        .corner_radius(theme.corner_radius)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(
                RichText::new(repeat_label(i18n, repeat_case))
                    .color(Color32::WHITE)
                    .family(egui::FontFamily::Name("montserrat-bold".into()))
                    .small(),
            );
        });
}
