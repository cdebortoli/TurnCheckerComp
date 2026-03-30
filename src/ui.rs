mod content;
mod pairing;
mod startup;
mod theme;

use eframe::egui;
use egui::RichText;
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
    pub fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let regular = egui::FontData::from_static(include_bytes!("../assets/fonts/Montserrat-Variable.ttf"))
            .tweak(egui::FontTweak {
                coords: egui::epaint::text::VariationCoords::new([("wght", 400.0)]),
                ..Default::default()
            });
        let bold = egui::FontData::from_static(include_bytes!("../assets/fonts/Montserrat-Variable.ttf"))
            .tweak(egui::FontTweak {
                coords: egui::epaint::text::VariationCoords::new([("wght", 700.0)]),
                ..Default::default()
            });

        fonts.font_data.insert(
            "montserrat_regular".to_owned(),
            regular.into(),
        );
        fonts.font_data.insert(
            "montserrat_bold".to_owned(),
            bold.into(),
        );

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "montserrat_regular".to_owned());
        }
        fonts.families.insert(
            egui::FontFamily::Name("montserrat-bold".into()),
            vec!["montserrat_bold".to_owned()],
        );

        ctx.set_fonts(fonts);
    }

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
        // Get theme once at the start
        let theme = theme::Theme::from_visuals(ui.visuals());

        // Check if ready and so that the server must be running
        self.startup
            .ensure_started(&mut self.runtime, self.pairing.pairing_state());
        // If server started but server_connection data not processed, configuring pairing system/view
        self.startup.sync_pairing_connection(&mut self.pairing);

        // UI - Full screen panel with theme background
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_primary)
                    .inner_margin(theme.spacing_lg),
            )
            .show_inside(ui, |ui| {
                ui.heading(RichText::new("Turn Checker Companion").color(theme.text_primary));

                if !self.startup.is_ready() {
                    self.startup.show_status(ui, &theme);
                } else if self.pairing.is_paired() {
                    self.content.show(ui);
                } else if self.startup.server_started() {
                    self.pairing.show_waiting(ui);
                } else {
                    ui.label(
                        RichText::new("Starting the local sync server...").color(theme.text_muted),
                    );
                }

                self.startup.show_restore_modal(
                    ui,
                    &mut self.runtime,
                    self.pairing.pairing_state(),
                    &theme,
                );
            });
    }
}
