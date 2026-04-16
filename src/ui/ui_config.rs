use super::{
    theme::Theme, TurnCheckerApp, CLASSIC_MIN_WINDOW_SIZE, CLASSIC_WINDOW_SIZE, MINIMAL_WINDOW_SIZE,
};
use crate::platform;
use eframe::egui;

impl TurnCheckerApp {
    pub fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let regular = egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Montserrat-Variable.ttf"
        ))
        .tweak(egui::FontTweak {
            coords: egui::epaint::text::VariationCoords::new([("wght", 400.0)]),
            ..Default::default()
        });
        let bold = egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Montserrat-Variable.ttf"
        ))
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
        eframe::icon_data::from_png_bytes(include_bytes!(
            "../../assets/icons/app_icon_ios_dark.png"
        ))
        .expect("embedded app icon should decode")
    }

    pub(super) fn classic_window_size() -> egui::Vec2 {
        egui::vec2(CLASSIC_WINDOW_SIZE[0], CLASSIC_WINDOW_SIZE[1])
    }

    fn classic_min_window_size() -> egui::Vec2 {
        egui::vec2(CLASSIC_MIN_WINDOW_SIZE[0], CLASSIC_MIN_WINDOW_SIZE[1])
    }

    fn minimal_window_size() -> egui::Vec2 {
        egui::vec2(MINIMAL_WINDOW_SIZE[0], MINIMAL_WINDOW_SIZE[1])
    }

    pub(super) fn with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    pub(super) fn sync_window_state(&mut self, ctx: &egui::Context) {
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

    pub(super) fn toggle_minimal_mode(&mut self, ctx: &egui::Context) {
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

    pub(super) fn toggle_always_on_top(&mut self, ctx: &egui::Context) {
        self.always_on_top = !self.always_on_top;
        self.apply_window_level(ctx);
        ctx.request_repaint();
    }

    pub(super) fn toggle_theme(ctx: &egui::Context, dark_mode: bool) {
        let visuals = if dark_mode {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        ctx.set_visuals(visuals);
        ctx.request_repaint();
    }

    pub(super) fn panel_fill_color(&self, theme: &Theme) -> egui::Color32 {
        if self.minimal_mode {
            egui::Color32::TRANSPARENT
        } else {
            theme.bg_primary
        }
    }

    pub(super) fn panel_margin(&self, theme: &Theme) -> i8 {
        if self.minimal_mode {
            0
        } else {
            theme.spacing_lg as i8
        }
    }
}
