use eframe::egui;
use qrcode::QrCode;
use tokio::runtime::Runtime;

use crate::channels::UiChannels;
use crate::{database, server};

pub struct TurnCheckerApp {
    runtime: Runtime,
    _channels: UiChannels,
    startup_state: StartupState,
    server_started: bool,
    pairing_state: server::PairingState,
    server_connection: Option<server::ServerConnectionInfo>,
    qr_texture: Option<egui::TextureHandle>,
}

enum StartupState {
    NeedsDecision { unsent_records: usize },
    Ready,
    Failed(String),
}

impl TurnCheckerApp {
    pub fn new(runtime: Runtime, channels: UiChannels) -> Self {
        let pairing_state = server::PairingState::new();
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
            pairing_state,
            server_connection: None,
            qr_texture: None,
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

        match self.runtime.block_on(server::spawn(self.pairing_state.clone())) {
            Ok(server_connection) => {
                self.server_started = true;
                self.server_connection = Some(server_connection);
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

    fn ensure_qr_texture(&mut self, ui: &egui::Ui) -> anyhow::Result<()> {
        if self.qr_texture.is_some() {
            return Ok(());
        }

        let qr_payload = self
            .server_connection
            .as_ref()
            .map(|info| info.qr_payload.as_str())
            .ok_or_else(|| anyhow::anyhow!("server connection info is not available"))?;

        let code = QrCode::new(qr_payload.as_bytes())?;
        let width = code.width();
        let pixels = code
            .to_colors()
            .into_iter()
            .map(|color| match color {
                qrcode::types::Color::Dark => egui::Color32::BLACK,
                qrcode::types::Color::Light => egui::Color32::WHITE,
            })
            .collect::<Vec<_>>();

        let image = egui::ColorImage::new([width, width], pixels);
        self.qr_texture = Some(ui.ctx().load_texture(
            "server-pairing-qr",
            image,
            egui::TextureOptions::NEAREST,
        ));
        Ok(())
    }

    fn show_waiting_for_pairing(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scan To Connect");
        ui.label("Open the iOS app and scan the QR code to configure the server address.");
        ui.add_space(12.0);

        if let Err(error) = self.ensure_qr_texture(ui) {
            ui.label("Failed to generate pairing QR code.");
            ui.monospace(error.to_string());
            return;
        }

        if let Some(texture) = &self.qr_texture {
            let image = egui::Image::new(texture).fit_to_exact_size(egui::vec2(280.0, 280.0));
            ui.add(image);
        }

        if let Some(server_connection) = &self.server_connection {
            ui.add_space(12.0);
            ui.label("Server URL");
            ui.monospace(&server_connection.base_url);
        }
    }

    fn show_connected_view(&self, ui: &mut egui::Ui) {
        ui.heading("Device Connected");
        ui.label("The iOS app is now paired with this server.");
        ui.label("Future content will appear in this view.");
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
                if self.server_started && self.pairing_state.is_paired() {
                    self.show_connected_view(ui);
                } else if self.server_started {
                    self.show_waiting_for_pairing(ui);
                } else {
                    ui.label("Starting the local sync server...");
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
