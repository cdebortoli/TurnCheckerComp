mod components;
mod content;
mod pairing;
mod startup;
mod theme;

use eframe::egui;
use egui::RichText;
use tokio::runtime::Runtime;

use crate::{channels::UiChannels, i18n::I18n, platform, server};

use self::components::round_icon_button::round_icon_button;
use self::content::MainContentView;
use self::pairing::PairingView;
use self::startup::StartupController;

const CLASSIC_WINDOW_SIZE: [f32; 2] = [960.0, 640.0];
const CLASSIC_MIN_WINDOW_SIZE: [f32; 2] = [640.0, 480.0];
const MINIMAL_WINDOW_SIZE: [f32; 2] = [72.0, 72.0];
const TITLE_BAR_BUTTON_SIZE: f32 = 20.0;
const MINIMAL_MODE_BUTTON_SIZE: f32 = 28.0;
const MINIMAL_MODE_BUTTON_BG_ALPHA: u8 = 196;

pub struct TurnCheckerApp {
    runtime: Runtime,
    _channels: UiChannels,
    i18n: I18n,
    startup: StartupController,
    pairing: PairingView,
    content: MainContentView,
    push_notification_client: server::PushNotificationClient,
    minimal_mode: bool,
    always_on_top: bool,
    classic_window_size: egui::Vec2,
    classic_window_position: Option<egui::Pos2>,
    minimal_window_position: Option<egui::Pos2>,
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

    pub fn new(
        runtime: Runtime,
        repaint_ctx: egui::Context,
        channels: UiChannels,
        i18n: I18n,
    ) -> Self {
        let mut repaint_refresh_rx = channels.content_refresh_rx.clone();
        let push_notification_client = server::PushNotificationClient::new();
        runtime.spawn(async move {
            while repaint_refresh_rx.changed().await.is_ok() {
                repaint_ctx.request_repaint();
            }
        });

        Self {
            runtime,
            _channels: channels.clone(),
            i18n: i18n.clone(),
            startup: StartupController::new(
                channels.content_refresh_tx.clone(),
                push_notification_client.clone(),
                i18n.clone(),
            ),
            pairing: PairingView::new(i18n.clone()),
            content: MainContentView::new(channels.content_refresh_rx.clone(), i18n),
            push_notification_client,
            minimal_mode: false,
            always_on_top: false,
            classic_window_size: Self::classic_window_size(),
            classic_window_position: None,
            minimal_window_position: None,
        }
    }

    pub fn native_options(title: &str) -> eframe::NativeOptions {
        let mut options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(CLASSIC_WINDOW_SIZE)
                .with_min_inner_size(CLASSIC_MIN_WINDOW_SIZE)
                .with_title(title)
                .with_icon(Self::app_icon())
                .with_decorations(true)
                .with_transparent(true)
                .with_titlebar_shown(true)
                .with_title_shown(true),
            ..Default::default()
        };

        platform::configure_native_options(&mut options);
        options
    }

    fn app_icon() -> egui::IconData {
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/app_icon_ios_dark.png"))
            .expect("embedded app icon should decode")
    }

    fn classic_window_size() -> egui::Vec2 {
        egui::vec2(CLASSIC_WINDOW_SIZE[0], CLASSIC_WINDOW_SIZE[1])
    }

    fn classic_min_window_size() -> egui::Vec2 {
        egui::vec2(CLASSIC_MIN_WINDOW_SIZE[0], CLASSIC_MIN_WINDOW_SIZE[1])
    }

    fn minimal_window_size() -> egui::Vec2 {
        egui::vec2(MINIMAL_WINDOW_SIZE[0], MINIMAL_WINDOW_SIZE[1])
    }

    fn with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    fn sync_window_state(&mut self, ctx: &egui::Context) {
        if let Some(position) = ctx.input(|i| i.viewport().outer_rect.map(|rect| rect.min)) {
            if self.minimal_mode {
                self.minimal_window_position = Some(position);
            } else {
                self.classic_window_position = Some(position);
            }
        }

        if self.minimal_mode {
            return;
        }

        if let Some(size) = ctx.input(|i| i.viewport().inner_rect.map(|rect| rect.size())) {
            let min_size = Self::classic_min_window_size();
            if size.x >= min_size.x && size.y >= min_size.y {
                self.classic_window_size = size;
            }
        }
    }

    fn restore_window_position(&self, ctx: &egui::Context, position: Option<egui::Pos2>) {
        if let Some(position) = position {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
        }
    }

    fn set_minimal_mode(&mut self, ctx: &egui::Context, minimal_mode: bool) {
        if self.minimal_mode == minimal_mode {
            return;
        }

        self.sync_window_state(ctx);

        if minimal_mode {
            self.sync_window_state(ctx);
            let minimal_size = Self::minimal_window_size();
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(minimal_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(minimal_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(minimal_size));
            self.restore_window_position(ctx, self.minimal_window_position);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
                Self::classic_min_window_size(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(egui::Vec2::INFINITY));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.classic_window_size));
            self.restore_window_position(ctx, self.classic_window_position);
        }

        self.minimal_mode = minimal_mode;
        ctx.request_repaint();
    }

    fn toggle_minimal_mode(&mut self, ctx: &egui::Context) {
        self.set_minimal_mode(ctx, !self.minimal_mode);
    }

    fn apply_window_level(&self, ctx: &egui::Context) {
        let level = if self.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
    }

    fn toggle_always_on_top(&mut self, ctx: &egui::Context) {
        self.always_on_top = !self.always_on_top;
        self.apply_window_level(ctx);
        ctx.request_repaint();
    }

    fn toggle_theme(ctx: &egui::Context, dark_mode: bool) {
        let visuals = if dark_mode {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        ctx.set_visuals(visuals);
        ctx.request_repaint();
    }

    fn show_title_bar(&mut self, ui: &mut egui::Ui, theme: &theme::Theme) {
        let title_bar = ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!(
                    "../assets/icons/app_icon_ios_dark.png"
                ))
                .fit_to_exact_size(egui::vec2(24.0, 24.0)),
            );
            ui.heading(RichText::new(self.i18n.t("app-title")).color(theme.text_primary));

            let button_size = egui::vec2(20.0, 20.0);
            let available_width = ui.available_width().max(button_size.x);
            ui.allocate_ui_with_layout(
                egui::vec2(available_width, button_size.y),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if Self::show_theme_toggle_button(ui, theme, &self.i18n).clicked() {
                        Self::toggle_theme(ui.ctx(), ui.visuals().dark_mode);
                    }

                    ui.add_space(theme.spacing_sm);

                    if Self::show_always_on_top_button(ui, theme, self.always_on_top, &self.i18n)
                        .clicked()
                    {
                        self.toggle_always_on_top(ui.ctx());
                    }

                    ui.add_space(theme.spacing_sm);

                    if Self::show_minimal_mode_button(ui, theme, self.minimal_mode, &self.i18n)
                        .clicked()
                    {
                        self.toggle_minimal_mode(ui.ctx());
                    }
                },
            );
        });

        if title_bar.response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }

    fn show_theme_toggle_button(
        ui: &mut egui::Ui,
        theme: &theme::Theme,
        i18n: &I18n,
    ) -> egui::Response {
        ui.add(round_icon_button(
            theme,
            egui::vec2(TITLE_BAR_BUTTON_SIZE, TITLE_BAR_BUTTON_SIZE),
            false,
            None,
            |painter, rect, fill, _icon_color| {
                let center = rect.center();
                let moon_radius = rect.width() * 0.18;
                painter.circle_filled(center, moon_radius, theme.accent);
                painter.circle_filled(
                    center + egui::vec2(moon_radius * 0.95, -moon_radius * 0.3),
                    moon_radius,
                    fill,
                );
            },
        ))
        .on_hover_text(i18n.t("app-theme-toggle-tooltip"))
    }

    fn show_always_on_top_button(
        ui: &mut egui::Ui,
        theme: &theme::Theme,
        active: bool,
        i18n: &I18n,
    ) -> egui::Response {
        ui.add(round_icon_button(
            theme,
            egui::vec2(TITLE_BAR_BUTTON_SIZE, TITLE_BAR_BUTTON_SIZE),
            active,
            None,
            |painter, rect, _fill, icon_color| {
                let stem_top = rect.center() + egui::vec2(0.0, -rect.height() * 0.18);
                let stem_bottom = rect.center() + egui::vec2(0.0, rect.height() * 0.18);
                let stroke = egui::Stroke::new(1.4, icon_color);

                painter.line_segment([stem_top, stem_bottom], stroke);
                painter.line_segment(
                    [
                        stem_bottom,
                        stem_bottom + egui::vec2(0.0, rect.height() * 0.14),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        rect.center() + egui::vec2(-rect.width() * 0.18, -rect.height() * 0.02),
                        rect.center() + egui::vec2(rect.width() * 0.18, -rect.height() * 0.02),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        rect.center() + egui::vec2(-rect.width() * 0.18, -rect.height() * 0.02),
                        rect.center() + egui::vec2(0.0, -rect.height() * 0.26),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        rect.center() + egui::vec2(rect.width() * 0.18, -rect.height() * 0.02),
                        rect.center() + egui::vec2(0.0, -rect.height() * 0.26),
                    ],
                    stroke,
                );
            },
        ))
        .on_hover_text(if active {
            i18n.t("app-always-on-top-disable-tooltip")
        } else {
            i18n.t("app-always-on-top-enable-tooltip")
        })
    }

    fn show_minimal_mode_button(
        ui: &mut egui::Ui,
        theme: &theme::Theme,
        minimal_mode: bool,
        i18n: &I18n,
    ) -> egui::Response {
        let size = if minimal_mode {
            egui::vec2(MINIMAL_MODE_BUTTON_SIZE, MINIMAL_MODE_BUTTON_SIZE)
        } else {
            egui::vec2(TITLE_BAR_BUTTON_SIZE, TITLE_BAR_BUTTON_SIZE)
        };
        let fill_override = minimal_mode
            .then(|| Self::with_alpha(theme.bg_turn_card, MINIMAL_MODE_BUTTON_BG_ALPHA));

        ui.add(round_icon_button(
            theme,
            size,
            minimal_mode,
            fill_override,
            |painter, rect, _fill, icon_color| {
                let stroke = egui::Stroke::new(1.4, icon_color);
                let inset = if minimal_mode {
                    rect.width() * 0.23
                } else {
                    rect.width() * 0.32
                };
                let icon_rect = rect.shrink(inset);

                painter.rect_stroke(icon_rect, 2.0, stroke, egui::StrokeKind::Inside);

                if !minimal_mode {
                    let inner = icon_rect.shrink(rect.width() * 0.12);
                    painter.rect_stroke(
                        inner,
                        2.0,
                        egui::Stroke::new(1.0, icon_color),
                        egui::StrokeKind::Inside,
                    );
                }
            },
        ))
        .on_hover_text(if minimal_mode {
            i18n.t("app-minimal-mode-disable-tooltip")
        } else {
            i18n.t("app-minimal-mode-enable-tooltip")
        })
    }

    fn show_minimal_view(&mut self, ui: &mut egui::Ui, theme: &theme::Theme) {
        let drag_response = ui.interact(
            ui.max_rect(),
            ui.id().with("minimal_drag_area"),
            egui::Sense::click_and_drag(),
        );
        if drag_response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                if Self::show_minimal_mode_button(ui, theme, self.minimal_mode, &self.i18n)
                    .clicked()
                {
                    self.toggle_minimal_mode(ui.ctx());
                }
            },
        );
    }

    fn panel_fill_color(&self, theme: &theme::Theme) -> egui::Color32 {
        if self.minimal_mode {
            egui::Color32::TRANSPARENT
        } else {
            theme.bg_primary
        }
    }

    fn panel_margin(&self, theme: &theme::Theme) -> i8 {
        if self.minimal_mode {
            0
        } else {
            theme.spacing_lg as i8
        }
    }
}

impl eframe::App for TurnCheckerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Get theme once at the start
        let theme = theme::Theme::from_visuals(ui.visuals());
        self.sync_window_state(ui.ctx());

        // Check if ready and so that the server must be running
        self.startup
            .ensure_started(&mut self.runtime, self.pairing.pairing_state());
        // If server started but server_connection data not processed, configuring pairing system/view
        self.startup.sync_pairing_connection(&mut self.pairing);

        // UI - Full screen panel with theme background
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.panel_fill_color(&theme))
                    .inner_margin(self.panel_margin(&theme)),
            )
            .show_inside(ui, |ui| {
                if self.minimal_mode {
                    self.show_minimal_view(ui, &theme);
                } else {
                    self.show_title_bar(ui, &theme);
                    ui.add_space(theme.spacing_md);

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
                            RichText::new(self.i18n.t("app-server-starting"))
                                .color(theme.text_muted),
                        );
                    }

                    self.startup.show_restore_modal(
                        ui,
                        &mut self.runtime,
                        self.pairing.pairing_state(),
                        &theme,
                    );
                }
            });
    }
}
