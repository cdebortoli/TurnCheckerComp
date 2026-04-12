use super::{ContentMode, MainContentView, SourceChecksConfig};
use crate::models::check_source_type::CheckSourceType;
use crate::ui::theme::Theme;
use eframe::egui::{self, RichText};

impl MainContentView {
    pub(super) fn show_top_bar(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.horizontal(|ui| match self.mode {
            ContentMode::General | ContentMode::WaitingForNextTurn => {
                ui.add_enabled_ui(!self.is_waiting_for_next_turn(), |ui| {
                    self.show_next_turn_button(ui, theme);
                    self.show_mode_button(ui, theme, "New Check", ContentMode::NewCheck);
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Game's turns checks",
                        CheckSourceType::Game,
                    );
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Game's checks",
                        CheckSourceType::GlobalGame,
                    );
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Template's checks",
                        CheckSourceType::Blueprint,
                    );
                    self.show_mode_button(ui, theme, "Comments", ContentMode::Comments);
                    self.show_restart_button(ui, theme);
                });
            }
            _ => {
                if ui
                    .button(RichText::new("Back").color(theme.text_primary))
                    .clicked()
                {
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
            }
        });
    }

    fn show_next_turn_button(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let button = egui::Button::new(RichText::new("Next turn").color(theme.text_primary))
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.handle_new_turn_click();
        }
    }

    fn show_mode_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &str,
        target_mode: ContentMode,
    ) {
        let button = egui::Button::new(RichText::new(label).color(theme.text_primary))
            .fill(if self.mode == target_mode {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.mode = target_mode;
            self.error_message = None;
        }
    }

    fn show_source_checks_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &'static str,
        source: CheckSourceType,
    ) {
        let is_active = self.mode == ContentMode::SourceChecks
            && self
                .source_checks_config
                .as_ref()
                .is_some_and(|config| config.title == label && config.source == source);
        let button = egui::Button::new(RichText::new(label).color(theme.text_primary))
            .fill(if is_active {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.mode = ContentMode::SourceChecks;
            self.source_checks_config = Some(SourceChecksConfig {
                title: label,
                source,
            });
            self.error_message = None;
            self.needs_reload = true;
        }
    }

    fn show_restart_button(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let button = egui::Button::new(RichText::new("Restart").color(theme.text_primary))
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.handle_restart_click();
        }
    }
}
