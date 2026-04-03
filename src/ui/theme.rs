use egui::Color32;

pub struct Theme {
    // Backgrounds
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_list: Color32,
    pub bg_list_element: Color32,
    pub bg_modal_overlay: Color32,
    pub bg_turn_card: Color32,

    // Text
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,

    // Accent / interactive
    pub accent: Color32,
    pub accent_hover: Color32,
    pub destructive: Color32,
    pub success: Color32,
    pub warning: Color32,

    // Check source indicator colors
    pub source_blueprint: Color32,
    pub source_game: Color32,
    pub source_global: Color32,
    pub source_turn: Color32,

    // Repeat type colors (matching iOS)
    pub repeat_everytime: Color32,
    pub repeat_conditional: Color32,
    pub repeat_specific: Color32,
    pub repeat_until: Color32,

    // Badge
    pub badge_default: Color32,

    // Spacing
    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,
    pub corner_radius: f32,
    pub card_padding: f32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            // Backgrounds
            bg_primary: Color32::from_rgb(28, 28, 30),
            bg_secondary: Color32::from_rgb(44, 44, 46),
            bg_list: Color32::from_rgb(36, 36, 38),
            bg_list_element: Color32::from_rgb(54, 54, 56),
            bg_modal_overlay: Color32::from_rgba_premultiplied(0, 0, 0, 180),
            bg_turn_card: Color32::from_rgb(44, 44, 46),

            // Text
            text_primary: Color32::from_rgb(242, 242, 247),
            text_secondary: Color32::from_rgb(174, 174, 178),
            text_muted: Color32::from_rgb(99, 99, 102),

            // Accent / interactive
            accent: Color32::from_rgb(222, 144, 48),
            accent_hover: Color32::from_rgb(64, 156, 255),
            destructive: Color32::from_rgb(255, 69, 58),
            success: Color32::from_rgb(48, 209, 88),
            warning: Color32::from_rgb(255, 159, 10),

            // Check source indicator colors (slightly muted for dark mode)
            source_blueprint: Color32::from_rgb(10, 132, 255),
            source_game: Color32::from_rgb(255, 55, 95),
            source_global: Color32::from_rgb(255, 159, 10),
            source_turn: Color32::from_rgb(48, 209, 88),

            // Repeat type colors (dark mode variants)
            repeat_everytime: Color32::from_rgb(10, 132, 255),
            repeat_conditional: Color32::from_rgb(191, 90, 242),
            repeat_specific: Color32::from_rgb(255, 159, 10),
            repeat_until: Color32::from_rgb(255, 55, 95),

            // Badge
            badge_default: Color32::from_rgb(99, 99, 102),

            // Spacing
            spacing_xs: 2.0,
            spacing_sm: 4.0,
            spacing_md: 8.0,
            spacing_lg: 16.0,
            spacing_xl: 24.0,
            corner_radius: 8.0,
            card_padding: 12.0,
        }
    }

    pub fn light() -> Self {
        Self {
            // Backgrounds
            bg_primary: Color32::from_rgb(242, 242, 247),
            bg_secondary: Color32::from_rgb(255, 255, 255),
            bg_list: Color32::from_rgb(242, 242, 247),
            bg_list_element: Color32::from_rgb(255, 255, 255),
            bg_modal_overlay: Color32::from_rgba_premultiplied(0, 0, 0, 100),
            bg_turn_card: Color32::from_rgb(255, 255, 255),

            // Text
            text_primary: Color32::from_rgb(0, 0, 0),
            text_secondary: Color32::from_rgb(60, 60, 67),
            text_muted: Color32::from_rgb(142, 142, 147),

            // Accent / interactive
            accent: Color32::from_rgb(222, 98, 48),
            accent_hover: Color32::from_rgb(0, 96, 210),
            destructive: Color32::from_rgb(255, 59, 48),
            success: Color32::from_rgb(52, 199, 89),
            warning: Color32::from_rgb(255, 149, 0),

            // Check source indicator colors (standard iOS colors)
            source_blueprint: Color32::from_rgb(0, 122, 255),
            source_game: Color32::from_rgb(255, 45, 85),
            source_global: Color32::from_rgb(255, 149, 0),
            source_turn: Color32::from_rgb(52, 199, 89),

            // Repeat type colors (standard iOS colors)
            repeat_everytime: Color32::from_rgb(52, 120, 246),
            repeat_conditional: Color32::from_rgb(175, 82, 222),
            repeat_specific: Color32::from_rgb(255, 149, 0),
            repeat_until: Color32::from_rgb(255, 45, 85),

            // Badge
            badge_default: Color32::from_rgb(142, 142, 147),

            // Spacing
            spacing_xs: 2.0,
            spacing_sm: 4.0,
            spacing_md: 8.0,
            spacing_lg: 16.0,
            spacing_xl: 24.0,
            corner_radius: 8.0,
            card_padding: 12.0,
        }
    }

    pub fn from_visuals(visuals: &egui::Visuals) -> Self {
        if visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }
}
