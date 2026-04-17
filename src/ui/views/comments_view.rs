use eframe::egui::{self, RichText};

use crate::i18n::I18n;
use crate::models::{Comment, CommentType};
use crate::ui::theme::Theme;
use crate::ui::ui_helpers::{
    find_comment_by_type, find_comment_by_type_mut, show_sent_status_icon,
};

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
        i18n: &I18n,
        comments: &mut [Comment],
    ) -> Option<CommentsAction> {
        egui::Frame::new()
            .fill(theme.bg_list)
            .corner_radius(theme.corner_radius)
            .inner_margin(theme.card_padding)
            .show(ui, |ui| {
                self.show_comments_content(ui, theme, i18n, comments)
            })
            .inner
    }

    fn show_comments_content(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        comments: &mut [Comment],
    ) -> Option<CommentsAction> {
        ui.heading(RichText::new(i18n.t("comments-title")).color(theme.text_primary));
        ui.add_space(theme.spacing_xs);
        ui.add_space(theme.spacing_md);

        self.show_comment_toolbar(ui, theme, i18n, comments);
        ui.add_space(theme.spacing_md);

        self.show_comment_editor(ui, theme, i18n, comments)
    }

    fn show_comment_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        i18n: &I18n,
        comments: &[Comment],
    ) {
        ui.horizontal(|ui| {
            self.show_comment_type_button(
                ui,
                theme,
                CommentType::Game,
                &i18n.t("comment-type-game"),
            );
            self.show_comment_type_button(
                ui,
                theme,
                CommentType::Turn,
                &i18n.t("comment-type-turn"),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_sent = find_comment_by_type(comments, self.selected_comment_type)
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
        i18n: &I18n,
        comments: &mut [Comment],
    ) -> Option<CommentsAction> {
        let Some(selected_comment) = find_comment_by_type_mut(comments, self.selected_comment_type)
        else {
            ui.label(RichText::new(i18n.t("comments-no-slot")).color(theme.text_muted));
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
                comment_type: self.selected_comment_type,
                content: selected_comment.content.clone(),
            })
        } else {
            None
        }
    }
}
