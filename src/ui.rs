use eframe::egui;
use tokio::runtime::Runtime;

use crate::channels::UiChannels;
use crate::{database, server};

pub struct TurnCheckerApp {
    runtime: Runtime,
    _channels: UiChannels,
    startup_state: StartupState,
    server_started: bool,
}

enum StartupState {
    NeedsDecision { unsent_records: usize },
    Ready,
    Failed(String),
}

impl TurnCheckerApp {
    pub fn new(runtime: Runtime, channels: UiChannels) -> Self {
        let startup_state = match database::inspect_startup_state() {
            Ok(database::DatabaseStartupState::Ready) => StartupState::Ready,
            Ok(database::DatabaseStartupState::NeedsUserDecision { unsent_records }) => {
                StartupState::NeedsDecision { unsent_records }
            }
            Err(error) => StartupState::Failed(error.to_string()),
        };

        Self {
            runtime,
            _channels: channels,
            startup_state,
            server_started: false,
        }
    }

    pub fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 640.0])
                .with_min_inner_size([640.0, 480.0])
                .with_title("Turn Checker Companion"),
            ..Default::default()
        }
    }

    fn continue_startup(&mut self) {
        if self.server_started {
            self.startup_state = StartupState::Ready;
            return;
        }

        match self.runtime.block_on(server::spawn()) {
            Ok(()) => {
                self.server_started = true;
                self.startup_state = StartupState::Ready;
            }
            Err(error) => self.startup_state = StartupState::Failed(error.to_string()),
        }
    }

    fn reset_database_and_continue(&mut self) {
        match database::reset_database() {
            Ok(()) => self.continue_startup(),
            Err(error) => self.startup_state = StartupState::Failed(error.to_string()),
        }
    }
}

impl eframe::App for TurnCheckerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if matches!(self.startup_state, StartupState::Ready) && !self.server_started {
            self.continue_startup();
        }

        ui.heading("Turn Checker Companion");
        ui.separator();

        match &self.startup_state {
            StartupState::NeedsDecision { unsent_records } => {
                ui.label(format!(
                    "The existing database contains {unsent_records} unsent record(s)."
                ));
                ui.label("Choose whether to keep it or recreate it.");
            }
            StartupState::Ready => {
                ui.label("Application startup completed.");
                if self.server_started {
                    ui.label("The local sync server is running.");
                }
            }
            StartupState::Failed(message) => {
                ui.label("Startup failed.");
                ui.monospace(message);
            }
        }

        let unsent_records = match &self.startup_state {
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
                        self.continue_startup();
                    }

                    if ui.button("Delete and recreate database").clicked() {
                        self.reset_database_and_continue();
                    }
                });
        }
    }
}
