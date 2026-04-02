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

    fn show_title_bar(&self, ui: &mut egui::Ui, theme: &theme::Theme) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Turn Checker Companion").color(theme.text_primary));

            let button_size = egui::vec2(20.0, 20.0);
            let available_width = ui.available_width().max(button_size.x);
            ui.allocate_ui_with_layout(
                egui::vec2(available_width, button_size.y),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if Self::show_theme_toggle_button(ui, theme).clicked() {
                        let visuals = if ui.visuals().dark_mode {
                            egui::Visuals::light()
                        } else {
                            egui::Visuals::dark()
                        };
                        ui.ctx().set_visuals(visuals);
                        ui.ctx().request_repaint();
                    }
                },
            );
        });
    }

    fn show_theme_toggle_button(ui: &mut egui::Ui, theme: &theme::Theme) -> egui::Response {
        let size = egui::vec2(20.0, 20.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let fill = if response.hovered() {
                theme.bg_turn_card
            } else {
                theme.bg_secondary
            };
            let stroke = egui::Stroke::new(1.0, theme.text_muted);
            let center = rect.center();
            let button_radius = rect.width() * 0.5;

            ui.painter().circle(center, button_radius, fill, stroke);

            let moon_radius = rect.width() * 0.18;
            let moon_color = theme.accent;
            ui.painter().circle_filled(center, moon_radius, moon_color);
            ui.painter().circle_filled(
                center + egui::vec2(moon_radius * 0.95, -moon_radius * 0.3),
                moon_radius,
                fill,
            );
        }

        response.on_hover_text("Toggle light/dark mode")
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
                self.show_title_bar(ui, &theme);

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
