use super::{ContentAction, ContentMode, MainContentView};
use crate::i18n::I18nValue;
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
                self.show_top_bar(ui, theme);
                ui.add_space(theme.spacing_md);
                self.show_error_message(ui, theme);
                self.show_active_content(ui, theme);
            });
        self.show_new_turn_confirmation(ui, theme, action);
        self.show_restart_confirmation(ui, theme, action);
    }

    fn show_new_turn_confirmation(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let Some(unchecked_mandatory_checks) = self.new_turn_confirmation_open else {
            return;
        };

        let ctx = ui.ctx().clone();
        egui::Window::new(self.i18n.t("dialog-new-turn-title"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(&ctx, |ui| {
                if unchecked_mandatory_checks > 0 {
                    ui.label(
                        RichText::new(self.i18n.tr(
                            "dialog-new-turn-pending-message",
                            &[("count", I18nValue::from(unchecked_mandatory_checks))],
                        ))
                        .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new(self.i18n.t("dialog-new-turn-blocked-message"))
                            .color(theme.text_muted),
                    );
                } else {
                    ui.label(
                        RichText::new(self.i18n.t("dialog-new-turn-confirm-message"))
                            .color(theme.text_primary),
                    );
                }

                ui.add_space(theme.spacing_md);

                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("action-cancel")).clicked() {
                        self.new_turn_confirmation_open = None;
                    }

                    if unchecked_mandatory_checks == 0
                        && ui.button(self.i18n.t("action-next-turn")).clicked()
                    {
                        self.new_turn_confirmation_open = None;
                        match self.request_new_turn() {
                            Ok(()) => *action = Some(ContentAction::NewTurnNotifRequested),
                            Err(error) => self.error_message = Some(error),
                        }
                    }
                });
            });
    }

    fn show_restart_confirmation(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let Some(unsent_records) = self.restart_confirmation_unsent_checks else {
            return;
        };

        let ctx = ui.ctx().clone();
        egui::Window::new(self.i18n.t("dialog-restart-title"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(&ctx, |ui| {
                if unsent_records > 0 {
                    ui.label(
                        RichText::new(self.i18n.tr(
                            "dialog-restart-unsent-message",
                            &[("count", I18nValue::from(unsent_records))],
                        ))
                        .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new(self.i18n.t("dialog-restart-confirm-message"))
                            .color(theme.text_muted),
                    );
                } else {
                    ui.label(
                        RichText::new(self.i18n.t("dialog-restart-confirm-message"))
                            .color(theme.text_primary),
                    );
                }

                ui.add_space(theme.spacing_md);

                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("action-cancel")).clicked() {
                        self.restart_confirmation_unsent_checks = None;
                    }

                    if ui.button(self.i18n.t("action-restart")).clicked() {
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
                    &self.i18n,
                    self.current_session.as_ref(),
                    &self.checks,
                    &self.tags,
                );
                if let Some(action) = action {
                    self.handle_checklist_action(action);
                }
            }
            ContentMode::WaitingForNextTurn => {
                self.next_turn_waiting_view.show(ui, theme, &self.i18n);
            }
            ContentMode::NewCheck => {
                let action = self.new_check_view.show(ui, theme, &self.i18n, &self.tags);
                if let Some(action) = action {
                    self.handle_new_check_action(action);
                }
            }
            ContentMode::SourceChecks => {
                if let Some(config) = self.source_checks_config.as_ref() {
                    let title = self.i18n.t(config.title_key);
                    self.source_checks_view
                        .show(ui, theme, &self.i18n, &title, &self.source_checks, &self.tags);
                }
            }
            ContentMode::Comments => {
                let action = self
                    .comments_view
                    .show(ui, theme, &self.i18n, &mut self.comments);
                if let Some(action) = action {
                    self.handle_comments_action(action);
                }
            }
        }
    }
}
