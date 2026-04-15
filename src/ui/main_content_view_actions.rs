use super::checklist::ChecklistAction;
use super::comments::CommentsAction;
use super::new_check::NewCheckAction;
use super::source_checks::SourceChecksAction;
use super::{ContentMode, MainContentView};

impl MainContentView {
    pub(super) fn handle_new_turn_click(&mut self) {
        self.restart_confirmation_unsent_checks = None;

        if self.current_session.is_none() {
            self.new_turn_confirmation_open = None;
            self.error_message = Some(self.i18n.t("content-error-no-current-session"));
            return;
        }

        self.error_message = None;
        self.new_turn_confirmation_open = Some(self.count_unchecked_mandatory_turn_checks());
    }

    pub(super) fn handle_restart_click(&mut self) {
        self.new_turn_confirmation_open = None;

        match self.count_unsent_records() {
            Ok(unsent_records) => {
                self.restart_confirmation_unsent_checks = Some(unsent_records);
            }
            Err(error) => {
                self.error_message = Some(error);
            }
        }
    }

    pub(super) fn handle_checklist_action(&mut self, action: ChecklistAction) {
        match action {
            ChecklistAction::CheckToggled { check, is_checked } => {
                if let Err(error) = self.update_check_status(check, is_checked) {
                    self.error_message = Some(error);
                }
            }
            ChecklistAction::CheckSelected(check) => {
                self.new_check_view.start_editing(&check);
                self.mode = ContentMode::NewCheck;
                self.error_message = None;
            }
        }
    }

    pub(super) fn handle_comments_action(&mut self, action: CommentsAction) {
        match action {
            CommentsAction::CommentChanged {
                comment_type,
                content,
            } => match self.update_comment_content(comment_type, content) {
                Ok(()) => self.error_message = None,
                Err(error) => self.error_message = Some(error),
            },
        }
    }

    pub(super) fn handle_new_check_action(&mut self, action: NewCheckAction) {
        match action {
            NewCheckAction::Cancelled => {
                self.new_check_view.reset();
                self.mode = ContentMode::General;
                self.error_message = None;
            }
            NewCheckAction::SaveNewRequested(check) => match self.insert_new_check(check) {
                Ok(()) => {
                    self.new_check_view.reset();
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
                Err(error) => self.error_message = Some(error),
            },
            NewCheckAction::SaveExistingRequested(check) => {
                match self.update_existing_check(check) {
                    Ok(()) => {
                        self.new_check_view.reset();
                        self.mode = ContentMode::General;
                        self.error_message = None;
                    }
                    Err(error) => self.error_message = Some(error),
                }
            }
            NewCheckAction::ValidationFailed(error) => {
                self.error_message = Some(error);
            }
        }
    }

    pub(super) fn handle_source_checks_action(&mut self, action: SourceChecksAction) {
        match action {
            SourceChecksAction::CheckSelected(check) => {
                self.new_check_view.start_editing(&check);
                self.mode = ContentMode::NewCheck;
                self.error_message = None;
            }
        }
    }

    fn count_unchecked_mandatory_turn_checks(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| {
                check.source == crate::models::check_source_type::CheckSourceType::Turn
                    && check.is_mandatory
                    && !check.is_checked
            })
            .count()
    }
}
