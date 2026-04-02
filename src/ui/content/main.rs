use eframe::egui::{self, RichText};

use super::{ContentAction, ContentMode, MainContentView};
use crate::ui::theme::Theme;

impl MainContentView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<ContentAction> {
        let theme = Theme::from_visuals(ui.visuals());
        let mut action = None;
        self.sync_external_content_updates();
        self.reload_checks_if_needed();
        self.show_root_frame(ui, &theme, &mut action);
        action
    }

    fn show_root_frame(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        egui::Frame::new()
            .fill(theme.bg_primary)
            .inner_margin(theme.spacing_lg)
            .show(ui, |ui| {
                self.show_top_bar(ui, theme, action);
                ui.add_space(theme.spacing_md);
                self.show_error_message(ui, theme);
                self.show_active_content(ui, theme);
            });
        self.show_restart_confirmation(ui, theme, action);
    }

    fn show_top_bar(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        ui.horizontal(|ui| {
            if self.mode != ContentMode::General {
                if ui
                    .button(RichText::new("Back").color(theme.text_primary))
                    .clicked()
                {
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
            } else {
                self.show_mode_button(ui, theme, "New Check", ContentMode::NewCheck);
                self.show_mode_button(ui, theme, "Comments", ContentMode::Comments);
                self.show_restart_button(ui, theme, action);
            }
        });
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

    fn show_restart_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let button = egui::Button::new(RichText::new("Restart").color(theme.text_primary))
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            *action = self.handle_restart_click();
        }
    }

    fn show_restart_confirmation(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let Some(unsent_checks) = self.restart_confirmation_unsent_checks else {
            return;
        };

        let ctx = ui.ctx().clone();
        egui::Window::new("Restart")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "The database contains {unsent_checks} unsent check(s)."
                    ))
                    .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(
                        "Restarting will delete and recreate the database, then return to the pairing screen.",
                    )
                    .color(theme.text_muted),
                );
                ui.add_space(theme.spacing_md);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.restart_confirmation_unsent_checks = None;
                    }

                    if ui.button("Restart").clicked() {
                        self.restart_confirmation_unsent_checks = None;
                        *action = Some(ContentAction::RestartRequested);
                    }
                });
            });
    }

    fn show_error_message(&self, ui: &mut egui::Ui, theme: &Theme) {
        if let Some(error) = &self.error_message {
            ui.label(RichText::new(error).color(theme.destructive));
            ui.add_space(theme.spacing_md);
        }
    }

    fn show_active_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        match self.mode {
            ContentMode::General => self.show_general_content(ui, theme),
            ContentMode::NewCheck => self.show_new_check_content(ui, theme),
            ContentMode::Comments => self.show_comments_content(ui, theme),
        }
    }
}
