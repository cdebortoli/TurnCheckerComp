mod content;
mod pairing;
mod startup;
mod theme;

use eframe::egui;
use tokio::runtime::Runtime;

use crate::channels::UiChannels;

use self::content::MainContentView;
use self::pairing::PairingView;
use self::startup::StartupController;

pub struct TurnCheckerApp {
    runtime: Runtime,
    _channels: UiChannels,
    startup: StartupController,
    pairing: PairingView,
    content: MainContentView,
}

impl TurnCheckerApp {
    pub fn new(runtime: Runtime, channels: UiChannels) -> Self {
        Self {
            runtime,
            _channels: channels,
            startup: StartupController::new(),
            pairing: PairingView::new(),
            content: MainContentView::new(),
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
}

impl eframe::App for TurnCheckerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Check if ready and so that the server must be running
        self.startup
            .ensure_started(&mut self.runtime, self.pairing.pairing_state());
        // If server started but server_connection data not processed, configuring pairing system/view
        self.startup.sync_pairing_connection(&mut self.pairing);

        // UI
        ui.heading("Turn Checker Companion");
        ui.separator();

        if !self.startup.is_ready() {
            self.startup.show_status(ui); // Database restore warning
        } else if self.pairing.is_paired() {
            self.content.show(ui); // Show content
        } else if self.startup.server_started() {
            self.pairing.show_waiting(ui); // Pairing process
        } else {
            ui.label("Starting the local sync server..."); // Error
        }

        // Database restore process
        self.startup
            .show_restore_modal(ui, &mut self.runtime, self.pairing.pairing_state());
        // Update pairing if server was started
        self.startup.sync_pairing_connection(&mut self.pairing);
    }
}
