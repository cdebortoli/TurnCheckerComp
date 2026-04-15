// Add a toggle to hide expired checks (compared to turn number)
use eframe::egui::{self, RichText};

use crate::i18n::{I18n, I18nValue};
use crate::models::{Check, Tag};
use crate::ui::components::check_cards::{CheckCardDisplayMode, CheckCardsAction, CheckCardsView};
use crate::ui::theme::Theme;

#[derive(Default)]
pub(super) struct SourceChecksView {
    check_cards_view: CheckCardsView,
}

pub(super) enum SourceChecksAction {
    CheckSelected(Check),
}

impl SourceChecksView {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        title: &str,
        checks: &[Check],
        tags: &[Tag],
    ) -> Option<SourceChecksAction> {
        self.show_header(ui, theme, title);

        if checks.is_empty() {
            self.show_empty_state(ui, theme, i18n, title);
            return None;
        }

        self.check_cards_view
            .show(
                ui,
                theme,
                i18n,
                checks,
                tags,
                CheckCardDisplayMode::ReadOnly,
            )
            .map(|action| match action {
                CheckCardsAction::CheckSelected(check) => SourceChecksAction::CheckSelected(check),
                CheckCardsAction::CheckToggled { .. } => {
                    unreachable!("read-only cards never toggle")
                }
            })
    }

    fn show_header(&self, ui: &mut egui::Ui, theme: &Theme, title: &str) {
        ui.heading(RichText::new(title).color(theme.text_primary));
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_state(&self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n, title: &str) {
        ui.label(
            RichText::new(i18n.tr("source-checks-empty", &[("title", I18nValue::from(title))]))
                .color(theme.text_muted),
        );
    }
}
