use crate::ui::theme::Theme;

fn show_round_icon_button(
    ui: &mut egui::Ui,
    theme: &Theme,
    size: egui::Vec2,
    active: bool,
    fill_override: Option<egui::Color32>,
    draw_icon: impl FnOnce(&egui::Painter, egui::Rect, egui::Color32, egui::Color32),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let fill = if let Some(fill) = fill_override {
            fill
        } else if active || response.hovered() {
            theme.bg_turn_card
        } else {
            theme.bg_secondary
        };
        let stroke = egui::Stroke::new(
            1.0,
            if active {
                theme.accent
            } else {
                theme.text_muted
            },
        );
        let center = rect.center();
        let button_radius = rect.width() * 0.5;

        ui.painter().circle(center, button_radius, fill, stroke);
        let icon_color = if active {
            theme.accent
        } else {
            theme.text_primary
        };
        draw_icon(ui.painter(), rect, fill, icon_color);
    }

    response
}

pub(crate) fn round_icon_button<'a>(
    theme: &'a Theme,
    size: egui::Vec2,
    active: bool,
    fill_override: Option<egui::Color32>,
    draw_icon: impl FnOnce(&egui::Painter, egui::Rect, egui::Color32, egui::Color32) + 'a,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        show_round_icon_button(ui, theme, size, active, fill_override, draw_icon)
    }
}
