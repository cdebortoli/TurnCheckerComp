use tokio::runtime::Runtime;

use crate::channels::UiChannels;

pub struct TurnCheckerApp {
    _runtime: Runtime,
    _channels: UiChannels,
}

impl TurnCheckerApp {
    pub fn new(runtime: Runtime, channels: UiChannels) -> Self {
        Self {
            _runtime: runtime,
            _channels: channels,
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
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}
}
