use eframe::egui;

pub struct MainContentView;

impl MainContentView {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Device Connected");
        ui.label("The iOS app is now paired with this server.");
        ui.label("Future content will appear in this view.");
    }
}
