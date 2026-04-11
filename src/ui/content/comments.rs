use eframe::egui::{self, RichText};

use super::helpers::{find_comment_by_type, find_comment_by_type_mut, show_sent_status_icon};
use crate::models::{Comment, CommentType};
use crate::ui::theme::Theme;

pub(super) struct CommentsView {
    selected_comment_type: CommentType,
}

pub(super) enum CommentsAction {
    CommentChanged {
        comment_type: CommentType,
        content: String,
    },
}

impl Default for CommentsView {
    fn default() -> Self {
        Self {
            selected_comment_type: CommentType::Game,
        }
    }
}

impl CommentsView {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        comments: &mut Vec<Comment>,
    ) -> Option<CommentsAction> {
        egui::Frame::new()
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| self.show_comments_content(ui, theme, comments))
            .inner
    }

    fn show_comments_content(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        comments: &mut Vec<Comment>,
    ) -> Option<CommentsAction> {
        ui.heading(RichText::new("Comments").color(theme.text_primary));
        ui.add_space(theme.spacing_md);

        self.show_comment_toolbar(ui, theme, comments);
        ui.add_space(theme.spacing_md);

        self.show_comment_editor(ui, theme, comments)
    }

    fn show_comment_toolbar(&mut self, ui: &mut egui::Ui, theme: &Theme, comments: &[Comment]) {
        ui.horizontal(|ui| {
            self.show_comment_type_button(ui, theme, CommentType::Game, "Game");
            self.show_comment_type_button(ui, theme, CommentType::Turn, "Turn");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_sent = find_comment_by_type(comments, self.selected_comment_type.clone())
                    .map(|comment| comment.is_sent)
                    .unwrap_or(true);
                show_sent_status_icon(ui, theme, is_sent);
            });
        });
    }

    fn show_comment_type_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        comment_type: CommentType,
        label: &str,
    ) {
        let button = egui::Button::new(RichText::new(label).color(theme.text_primary))
            .fill(if self.selected_comment_type == comment_type {
                theme.accent
            } else {
                theme.bg_list_element
            })
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.selected_comment_type = comment_type;
        }
    }

    fn show_comment_editor(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        comments: &mut [Comment],
    ) -> Option<CommentsAction> {
        let Some(selected_comment) =
            find_comment_by_type_mut(comments, self.selected_comment_type.clone())
        else {
            ui.label(RichText::new("No comment slot is available.").color(theme.text_muted));
            return None;
        };

        let response = ui.add(
            egui::TextEdit::multiline(&mut selected_comment.content)
                .desired_rows(16)
                .desired_width(f32::INFINITY)
                .background_color(theme.bg_list_element),
        );

        if response.changed() {
            Some(CommentsAction::CommentChanged {
                comment_type: self.selected_comment_type.clone(),
                content: selected_comment.content.clone(),
            })
        } else {
            None
        }
    }
}
