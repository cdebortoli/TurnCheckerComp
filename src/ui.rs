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
        let regular =
            egui::FontData::from_static(include_bytes!("../assets/fonts/Montserrat-Variable.ttf"))
                .tweak(egui::FontTweak {
                    coords: egui::epaint::text::VariationCoords::new([("wght", 400.0)]),
                    ..Default::default()
                });
        let bold =
            egui::FontData::from_static(include_bytes!("../assets/fonts/Montserrat-Variable.ttf"))
                .tweak(egui::FontTweak {
                    coords: egui::epaint::text::VariationCoords::new([("wght", 700.0)]),
                    ..Default::default()
                });

        fonts
            .font_data
            .insert("montserrat_regular".to_owned(), regular.into());
        fonts
            .font_data
            .insert("montserrat_bold".to_owned(), bold.into());

        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "montserrat_regular".to_owned());
        }
        fonts.families.insert(
            egui::FontFamily::Name("montserrat-bold".into()),
            vec!["montserrat_bold".to_owned()],
        );

        ctx.set_fonts(fonts);
    }

    pub fn new(runtime: Runtime, repaint_ctx: egui::Context, channels: UiChannels) -> Self {
        let mut repaint_refresh_rx = channels.content_refresh_rx.clone();
        //let repaint_ctx_for_task = repaint_ctx.clone();
        runtime.spawn(async move {
            while repaint_refresh_rx.changed().await.is_ok() {
                // repaint_ctx_for_task.request_repaint();
                repaint_ctx.request_repaint();
            }
        });

        Self {
            runtime,
            _channels: channels.clone(),
            startup: StartupController::new(channels.content_refresh_tx.clone()),
            pairing: PairingView::new(),
            content: MainContentView::new(channels.content_refresh_rx.clone()),
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
                    if let Some(action) = self.content.show(ui) {
                        self.handle_content_action(action);
                    }
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
