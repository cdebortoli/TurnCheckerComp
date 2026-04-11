use super::{ContentAction, ContentMode, MainContentView};
use crate::ui::theme::Theme;
use eframe::egui::{self, RichText};

impl MainContentView {
    pub(super) fn show_root_frame(
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
            ContentMode::General => {
                let action = self.checklist_view.show(
                    ui,
                    theme,
                    self.current_session.as_ref(),
                    &self.checks,
                    &self.tags,
                );
                if let Some(action) = action {
                    self.handle_checklist_action(action);
                }
            }
            ContentMode::WaitingForNextTurn => {
                self.next_turn_waiting_view.show(ui, theme);
            }
            ContentMode::NewCheck => {
                let action = self.new_check_view.show(ui, theme, &self.tags);
                if let Some(action) = action {
                    self.handle_new_check_action(action);
                }
            }
            ContentMode::SourceChecks => {
                if let Some(config) = self.source_checks_config.as_ref() {
                    self.source_checks_view.show(
                        ui,
                        theme,
                        config.title,
                        &self.source_checks,
                        &self.tags,
                    );
                }
            }
            ContentMode::Comments => {
                self.comments_view.show(ui, theme);
            }
        }
    }
}
