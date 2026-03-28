use eframe::egui::{self, RichText};

use super::MainContentView;
use crate::ui::theme::Theme;

impl MainContentView {
    pub(super) fn show_comments_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        egui::Frame::new()
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| {
                self.show_comments_placeholder(ui, theme);
            });
    }

    fn show_comments_placeholder(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading(RichText::new("Comments").color(theme.text_primary));
        ui.label(
            RichText::new("Comments content is planned for a future iteration.")
                .color(theme.text_secondary),
        );
    }
}
