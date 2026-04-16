mod components;
mod startup;
mod theme;
mod ui_config;
pub(crate) mod ui_helpers;
mod views;

use eframe::egui;
use egui::RichText;
use tokio::runtime::Runtime;

use self::views::main_content_view::ContentAction;
use crate::{channels::UiChannels, i18n::I18n, server};

use self::components::round_icon_button::round_icon_button;
use self::startup::StartupController;
use self::views::main_content_view::MainContentView;
use self::views::pairing_view::PairingView;

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
                    } else if self.startup.server_started {
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

impl TurnCheckerApp {
    pub fn handle_content_action(&mut self, action: ContentAction) {
        match action {
            ContentAction::NewTurnNotifRequested => self.new_turn_notif(),
            ContentAction::RestartRequested => self.restart_to_pairing(),
        }
    }

    fn new_turn_notif(&mut self) {
        let push_notification_client = self.push_notification_client.clone();
        match self
            .runtime
            .block_on(async move { push_notification_client.send_new_turn_notification().await })
        {
            Ok(()) => {}
            Err(error) => self.content.cancel_next_turn_wait(error.to_string()),
        }
    }

    fn restart_to_pairing(&mut self) {
        match crate::database::reset_database() {
            Ok(()) => {
                self.pairing.pairing_state().reset();
                self.content.prepare_for_restart();
                let next_version = (*self._channels.content_refresh_tx.borrow()).wrapping_add(1);
                let _ = self._channels.content_refresh_tx.send(next_version);
            }
            Err(error) => self.content.set_error_message(error.to_string()),
        }
    }
}
