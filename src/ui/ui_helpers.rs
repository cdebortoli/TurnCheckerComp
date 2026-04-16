use crate::models::{Check, Comment, CommentType, Tag};
use crate::ui::theme::Theme;
use eframe::egui;
use uuid::Uuid;

pub(super) fn apply_check_status_update(mut check: Check, is_checked: bool) -> Check {
    check.is_checked = is_checked;
    check.is_sent = false;
    check
}

pub(super) fn apply_comment_content_update(
    mut comment: Comment,
    content: impl Into<String>,
) -> Comment {
    comment.content = content.into();
    comment.is_sent = false;
    comment
}

pub(super) fn find_comment_by_type(
    comments: &[Comment],
    comment_type: CommentType,
) -> Option<&Comment> {
    comments
        .iter()
        .find(|comment| comment.comment_type == comment_type)
}

pub(super) fn find_comment_by_type_mut(
    comments: &mut [Comment],
    comment_type: CommentType,
) -> Option<&mut Comment> {
    comments
        .iter_mut()
        .find(|comment| comment.comment_type == comment_type)
}

pub(crate) fn find_tag_by_uuid(tags: &[Tag], tag_uuid: Option<Uuid>) -> Option<&Tag> {
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

pub(super) fn tag_fill_color(tag: &Tag, theme: &Theme) -> egui::Color32 {
    parse_hex_color(&tag.color).unwrap_or(theme.badge_default)
}

fn tag_text_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.text_color).unwrap_or(egui::Color32::WHITE)
}

pub(crate) fn show_sent_status_icon(ui: &mut egui::Ui, theme: &Theme, is_sent: bool) {
    let circle_color = if is_sent {
        theme.success.gamma_multiply(0.86)
    } else {
        theme.destructive.gamma_multiply(0.86)
    };
    let icon_color = theme.text_primary;
    let size = 20.0;
    let stroke = egui::Stroke::new(2.0, icon_color);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let center = rect.center();
    let radius = size * 0.5;

    ui.painter().circle_filled(center, radius, circle_color);

    if is_sent {
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 5.5, rect.center().y + 1.5),
                egui::pos2(rect.left() + 9.5, rect.bottom() - 6.0),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 9.5, rect.bottom() - 6.0),
                egui::pos2(rect.right() - 5.0, rect.top() + 6.0),
            ],
            stroke,
        );
    } else {
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                egui::pos2(rect.right() - 6.0, rect.bottom() - 6.0),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 6.0, rect.bottom() - 6.0),
                egui::pos2(rect.right() - 6.0, rect.top() + 6.0),
            ],
            stroke,
        );
    }
}

pub(crate) fn show_tag_capsule(ui: &mut egui::Ui, theme: &Theme, tag: &Tag) {
    egui::Frame::new()
        .fill(tag_fill_color(tag, theme))
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
