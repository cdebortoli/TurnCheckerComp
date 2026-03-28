use eframe::egui;
use tokio::runtime::Runtime;

use crate::{database, server};

use super::pairing::PairingView;

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

    pub fn show_status(&self, ui: &mut egui::Ui) {
        match &self.state {
            StartupState::NeedsDecision { unsent_records } => {
                ui.label(format!(
                    "The existing database contains {unsent_records} unsent record(s)."
                ));
                ui.label("Choose whether to keep it or recreate it.");
            }
            StartupState::Ready => {
                if !self.server_started {
                    ui.label("Starting the local sync server...");
                }
            }
            StartupState::Failed(message) => {
                ui.label("Startup failed.");
                ui.monospace(message);
            }
        }
    }

    pub fn show_restore_modal(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &mut Runtime,
        pairing_state: server::PairingState,
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
                    ui.label(format!(
                        "The current database contains {unsent_records} unsent record(s)."
                    ));
                    ui.label("Do you want to keep the current database?");
                    ui.add_space(8.0);

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
