use eframe::egui::{self, RichText};

use super::check_cards::{CheckCardDisplayMode, CheckCardsAction, CheckCardsView};
use crate::i18n::{I18n, I18nValue};
use crate::models::{Check, CurrentSession, Tag};
use crate::ui::theme::Theme;

#[derive(Default)]
pub(super) struct ChecklistView {
    check_cards_view: CheckCardsView,
}

pub(super) enum ChecklistAction {
    CheckToggled { check: Check, is_checked: bool },
}

impl ChecklistView {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        current_session: Option<&CurrentSession>,
        checks: &[Check],
        tags: &[Tag],
    ) -> Option<ChecklistAction> {
        self.show_checklist_header(ui, theme, i18n, current_session);

        if checks.is_empty() {
            self.show_empty_checklist(ui, theme, i18n);
            return None;
        }

        self.check_cards_view
            .show(ui, theme, i18n, checks, tags, CheckCardDisplayMode::Editable)
            .map(|action| match action {
                CheckCardsAction::CheckToggled { check, is_checked } => {
                    ChecklistAction::CheckToggled { check, is_checked }
                }
            })
    }

    fn show_checklist_header(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        current_session: Option<&CurrentSession>,
    ) {
        match current_session {
            Some(current_session) => {
                ui.label(RichText::new(&current_session.game_name).color(theme.text_primary));
                ui.label(
                    RichText::new(i18n.tr(
                        "checklist-turn-label",
                        &[("turn", I18nValue::from(current_session.turn_number))],
                    ))
                    .color(theme.text_secondary),
                );
            }
            None => {
                ui.label(RichText::new(i18n.t("checklist-current-turn")).color(theme.text_secondary));
            }
        }
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_checklist(&self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        ui.label(RichText::new(i18n.t("checklist-empty")).color(theme.text_muted));
    }
}
