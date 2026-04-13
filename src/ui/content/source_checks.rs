use eframe::egui::{self, RichText};

use super::check_cards::{CheckCardDisplayMode, CheckCardsView};
use crate::i18n::{I18n, I18nValue};
use crate::models::{Check, Tag};
use crate::ui::theme::Theme;

#[derive(Default)]
pub(super) struct SourceChecksView {
    check_cards_view: CheckCardsView,
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
    ) {
        self.show_header(ui, theme, title);

        if checks.is_empty() {
            self.show_empty_state(ui, theme, i18n, title);
            return;
        }

        let _ = self
            .check_cards_view
            .show(ui, theme, i18n, checks, tags, CheckCardDisplayMode::ReadOnly);
    }

    fn show_header(&self, ui: &mut egui::Ui, theme: &Theme, title: &str) {
        ui.heading(RichText::new(title).color(theme.text_primary));
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_state(&self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n, title: &str) {
        ui.label(
            RichText::new(i18n.tr(
                "source-checks-empty",
                &[("title", I18nValue::from(title))],
            ))
            .color(theme.text_muted),
        );
    }
}
