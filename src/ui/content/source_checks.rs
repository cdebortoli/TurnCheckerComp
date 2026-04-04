use eframe::egui::{self, RichText};

use super::check_cards::{CheckCardDisplayMode, CheckCardsView};
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
        title: &str,
        checks: &[Check],
        tags: &[Tag],
    ) {
        self.show_header(ui, theme, title);

        if checks.is_empty() {
            self.show_empty_state(ui, theme, title);
            return;
        }

        let _ = self
            .check_cards_view
            .show(ui, theme, checks, tags, CheckCardDisplayMode::ReadOnly);
    }

    fn show_header(&self, ui: &mut egui::Ui, theme: &Theme, title: &str) {
        ui.heading(RichText::new(title).color(theme.text_primary));
        ui.add_space(theme.spacing_md);
    }

    fn show_empty_state(&self, ui: &mut egui::Ui, theme: &Theme, title: &str) {
        ui.label(RichText::new(format!("No checks found for {title}.")).color(theme.text_muted));
    }
}
