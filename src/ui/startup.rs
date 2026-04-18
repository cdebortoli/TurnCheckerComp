use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use crate::{
    database,
    i18n::{I18n, I18nValue},
    server,
};

use super::theme::Theme;
use egui::RichText;

pub struct StartupController {
    i18n: I18n,
    state: StartupState,
    server_connection: Option<server::ServerConnectionInfo>,
    content_refresh_tx: watch::Sender<u64>,
    push_notification_client: server::PushNotificationClient,
}

enum StartupState {
    NeedsDecision { unsent_records: usize },
    Starting,
    Ready,
    Failed(String),
}

impl StartupController {
    pub fn new(
        content_refresh_tx: watch::Sender<u64>,
        push_notification_client: server::PushNotificationClient,
        i18n: I18n,
    ) -> Self {
        let state = match database::inspect_startup_state() {
            Ok(database::DatabaseStartupState::Ready) => StartupState::Starting,
            Ok(database::DatabaseStartupState::NeedsUserDecision { unsent_records }) => {
                StartupState::NeedsDecision { unsent_records }
            }
            Err(error) => StartupState::Failed(error.to_string()),
        };

        Self {
            i18n,
            state,
            server_connection: None,
            content_refresh_tx,
            push_notification_client,
        }
    }

    pub fn ensure_started(&mut self, runtime: &mut Runtime, pairing_state: &server::PairingState) {
        if matches!(self.state, StartupState::Starting) {
            self.startup_server(runtime, pairing_state);
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, StartupState::Ready)
    }

    pub fn take_server_connection(&mut self) -> Option<server::ServerConnectionInfo> {
        self.server_connection.take()
    }

    pub fn show_status(&self, ui: &mut egui::Ui, theme: &Theme) {
        egui::Frame::new()
            .fill(theme.bg_turn_card)
            .inner_margin(theme.spacing_md)
            .corner_radius(theme.corner_radius)
            .show(ui, |ui| match &self.state {
                StartupState::NeedsDecision { .. } => {
                    ui.label(RichText::new(self.i18n.t("startup-waiting")).color(theme.text_muted));
                }
                StartupState::Starting => {
                    ui.label(
                        RichText::new(self.i18n.t("app-server-starting")).color(theme.text_muted),
                    );
                }
                StartupState::Ready => {}
                StartupState::Failed(message) => {
                    ui.label(
                        RichText::new(self.i18n.t("startup-failed")).color(theme.text_primary),
                    );
                    ui.monospace(RichText::new(message).color(theme.destructive));
                }
            });
    }

    pub fn show_restore_modal(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &mut Runtime,
        pairing_state: &server::PairingState,
        theme: &Theme,
    ) {
        let unsent_records = match &self.state {
            StartupState::NeedsDecision { unsent_records } => Some(*unsent_records),
            _ => None,
        };

        if let Some(unsent_records) = unsent_records {
            let ctx = ui.ctx().clone();
            egui::Window::new(self.i18n.t("startup-unsent-data-title"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(theme.bg_secondary)
                        .inner_margin(theme.spacing_lg)
                        .corner_radius(theme.corner_radius),
                )
                .show(&ctx, |ui| {
                    ui.label(
                        RichText::new(self.i18n.tr(
                            "startup-unsent-data-message",
                            &[("count", I18nValue::from(unsent_records))],
                        ))
                        .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new(self.i18n.t("startup-keep-db-question"))
                            .color(theme.text_muted),
                    );
                    ui.add_space(theme.spacing_xs);
                    ui.add_space(theme.spacing_xl);

                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.t("startup-keep-db-button")).clicked() {
                            self.startup_server(runtime, pairing_state);
                        }

                        if ui.button(self.i18n.t("startup-reset-db-button")).clicked() {
                            self.reset_database_and_continue(runtime, pairing_state);
                        }
                    });
                });
        }
    }

    fn startup_server(&mut self, runtime: &mut Runtime, pairing_state: &server::PairingState) {
        if matches!(self.state, StartupState::Ready) {
            return;
        }

        self.state = StartupState::Starting;

        match runtime.block_on(server::spawn(
            pairing_state.clone(),
            self.content_refresh_tx.clone(),
            self.push_notification_client.clone(),
        )) {
            Ok(server_connection) => {
                self.server_connection = Some(server_connection);
                self.state = StartupState::Ready;
            }
            Err(error) => self.state = StartupState::Failed(error.to_string()),
        }
    }

    fn reset_database_and_continue(
        &mut self,
        runtime: &mut Runtime,
        pairing_state: &server::PairingState,
    ) {
        match database::reset_database() {
            Ok(()) => self.startup_server(runtime, pairing_state),
            Err(error) => self.state = StartupState::Failed(error.to_string()),
        }
    }
}
