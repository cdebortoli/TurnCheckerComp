use eframe::egui::{self, RichText};

use super::check_cards::{CheckCardDisplayMode, CheckCardsAction, CheckCardsView};
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
        current_session: Option<&CurrentSession>,
        checks: &[Check],
        tags: &[Tag],
    ) -> Option<ChecklistAction> {
        self.show_checklist_header(ui, theme, current_session);

        if checks.is_empty() {
            self.show_empty_checklist(ui, theme);
            return None;
        }

        self.check_cards_view
            .show(ui, theme, checks, tags, CheckCardDisplayMode::Editable)
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
        current_session: Option<&CurrentSession>,
    ) {
        match current_session {
            Some(current_session) => {
                ui.label(RichText::new(&current_session.game_name).color(theme.text_primary));
                ui.label(
                    RichText::new(format!("Turn {}", current_session.turn_number))
                        .color(theme.text_secondary),
                );
            }
            None => {
                ui.label(RichText::new("Current turn").color(theme.text_secondary));
            }
        }
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_checklist(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(RichText::new("No checks yet.").color(theme.text_muted));
    }
}
