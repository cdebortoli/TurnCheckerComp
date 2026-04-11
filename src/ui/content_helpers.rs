use crate::models::{Check, Tag};
use eframe::egui;
use uuid::Uuid;

pub(super) fn apply_check_status_update(mut check: Check, is_checked: bool) -> Check {
    check.is_checked = is_checked;
    check.is_sent = false;
    check
}

pub(super) fn find_tag_by_uuid(tags: &[Tag], tag_uuid: Option<Uuid>) -> Option<&Tag> {
    let tag_uuid = tag_uuid?;
    tags.iter().find(|tag| tag.uuid == tag_uuid)
}

fn parse_hex_color(value: &str) -> Option<egui::Color32> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(egui::Color32::from_rgb(red, green, blue))
}

pub(super) fn tag_fill_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.color).unwrap_or_else(|| egui::Color32::from_rgb(99, 99, 102))
}

fn tag_text_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.text_color).unwrap_or(egui::Color32::WHITE)
}

pub(super) fn show_tag_capsule(ui: &mut egui::Ui, tag: &Tag) {
    egui::Frame::new()
        .fill(tag_fill_color(tag))
        .corner_radius(999.0)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&tag.name)
                    .family(egui::FontFamily::Name("montserrat-bold".into()))
                    .color(tag_text_color(tag))
                    .small(),
            );
        });
}
