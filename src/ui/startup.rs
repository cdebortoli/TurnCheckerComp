use eframe::egui;
use tokio::runtime::Runtime;

use crate::{database, server};

use super::pairing::PairingView;
use super::theme::Theme;
use egui::RichText;

pub struct StartupController {
    state: StartupState,
    server_started: bool,
    server_connection: Option<server::ServerConnectionInfo>,
}

enum StartupState {
    NeedsDecision { unsent_records: usize },
    Ready,
    Failed(String),
}

impl StartupController {
    pub fn new() -> Self {
        let state = match database::inspect_startup_state() {
            Ok(database::DatabaseStartupState::Ready) => StartupState::Ready,
            Ok(database::DatabaseStartupState::NeedsUserDecision { unsent_records }) => {
                StartupState::NeedsDecision { unsent_records }
            }
            Err(error) => StartupState::Failed(error.to_string()),
        };

        Self {
            state,
            server_started: false,
            server_connection: None,
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
        match &self.state {
            StartupState::NeedsDecision { unsent_records } => {
                ui.label(RichText::new(format!(
                    "The existing database contains {unsent_records} unsent record(s)."
                )).color(theme.text_secondary));
                ui.label(RichText::new("Choose whether to keep it or recreate it.")
                    .color(theme.text_muted));
            }
            StartupState::Ready => {
                if !self.server_started {
                    ui.label(RichText::new("Starting the local sync server...")
                        .color(theme.text_muted));
                }
            }
            StartupState::Failed(message) => {
                ui.label(RichText::new("Startup failed.").color(theme.text_primary));
                ui.monospace(RichText::new(message).color(theme.destructive));
            }
        }
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
            egui::Window::new("Unsent data found")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label(RichText::new(format!(
                        "The current database contains {unsent_records} unsent record(s)."
                    )).color(theme.text_primary));
                    ui.label(RichText::new("Do you want to keep the current database?")
                        .color(theme.text_muted));
                    ui.add_space(theme.spacing_md);

                    if ui.button("Keep current database").clicked() {
                        self.continue_startup(runtime, pairing_state.clone());
                    }

                    if ui.button("Delete and recreate database").clicked() {
                        self.reset_database_and_continue(runtime, pairing_state.clone());
                    }
                });
        }
    }

    fn continue_startup(&mut self, runtime: &mut Runtime, pairing_state: server::PairingState) {
        if self.server_started {
            self.state = StartupState::Ready;
            return;
        }

        match runtime.block_on(server::spawn(pairing_state)) {
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
