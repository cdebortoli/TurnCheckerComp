use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use crate::{database, i18n::{I18n, I18nValue}, server};

use super::pairing::PairingView;
use super::theme::Theme;
use egui::RichText;

pub struct StartupController {
    i18n: I18n,
    state: StartupState,
    server_started: bool,
    server_connection: Option<server::ServerConnectionInfo>,
    content_refresh_tx: watch::Sender<u64>,
    push_notification_client: server::PushNotificationClient,
}

enum StartupState {
    NeedsDecision { unsent_records: usize },
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
            Ok(database::DatabaseStartupState::Ready) => StartupState::Ready,
            Ok(database::DatabaseStartupState::NeedsUserDecision { unsent_records }) => {
                StartupState::NeedsDecision { unsent_records }
            }
            Err(error) => StartupState::Failed(error.to_string()),
        };

        Self {
            i18n,
            state,
            server_started: false,
            server_connection: None,
            content_refresh_tx,
            push_notification_client,
        }
    }

    pub fn ensure_started(&mut self, runtime: &mut Runtime, pairing_state: server::PairingState) {
        if matches!(self.state, StartupState::Ready) && !self.server_started {
            self.continue_startup(runtime, pairing_state);
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, StartupState::Ready)
    }

    pub fn server_started(&self) -> bool {
        self.server_started
    }

    pub fn sync_pairing_connection(&mut self, pairing: &mut PairingView) {
        if let Some(server_connection) = self.server_connection.take() {
            pairing.set_server_connection(server_connection);
        }
    }

    pub fn show_status(&self, ui: &mut egui::Ui, theme: &Theme) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_turn_card)
                    .inner_margin(theme.spacing_md)
                    .corner_radius(theme.corner_radius),
            )
            .show_inside(ui, |ui| match &self.state {
                StartupState::NeedsDecision { .. } => {
                    ui.label(RichText::new(self.i18n.t("startup-waiting")).color(theme.text_muted));
                }
                StartupState::Ready => {
                    if !self.server_started {
                        ui.label(
                            RichText::new(self.i18n.t("app-server-starting"))
                                .color(theme.text_muted),
                        );
                    }
                }
                StartupState::Failed(message) => {
                    ui.label(RichText::new(self.i18n.t("startup-failed")).color(theme.text_primary));
                    ui.monospace(RichText::new(message).color(theme.destructive));
                }
            });
    }

    pub fn show_restore_modal(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &mut Runtime,
        pairing_state: server::PairingState,
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
                    ui.add_space(theme.spacing_md);

                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.t("startup-keep-db-button")).clicked() {
                            self.continue_startup(runtime, pairing_state.clone());
                        }

                        if ui.button(self.i18n.t("startup-reset-db-button")).clicked() {
                            self.reset_database_and_continue(runtime, pairing_state.clone());
                        }
                    });
                });
        }
    }

    fn continue_startup(&mut self, runtime: &mut Runtime, pairing_state: server::PairingState) {
        if self.server_started {
            self.state = StartupState::Ready;
            return;
        }

        match runtime.block_on(server::spawn(
            pairing_state,
            self.content_refresh_tx.clone(),
            self.push_notification_client.clone(),
        )) {
            Ok(server_connection) => {
                self.server_started = true;
                self.server_connection = Some(server_connection);
                self.state = StartupState::Ready;
            }
            Err(error) => self.state = StartupState::Failed(error.to_string()),
        }
    }

    fn reset_database_and_continue(
        &mut self,
        runtime: &mut Runtime,
        pairing_state: server::PairingState,
    ) {
        match database::reset_database() {
            Ok(()) => self.continue_startup(runtime, pairing_state),
            Err(error) => self.state = StartupState::Failed(error.to_string()),
        }
    }
}
