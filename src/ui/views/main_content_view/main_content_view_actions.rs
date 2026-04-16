use super::super::checklist_view::ChecklistAction;
use super::super::comments_view::CommentsAction;
use super::super::new_check_view::NewCheckAction;
use super::super::source_checks_view::SourceChecksAction;
use super::{Check, ContentMode, MainContentView};

impl MainContentView {
    fn open_new_check(&mut self, return_mode: ContentMode, check: &Check) {
        self.new_check_view.start_editing(check);
        self.new_check_return_mode = return_mode;
        self.mode = ContentMode::NewCheck;
        self.error_message = None;
    }

    fn close_new_check(&mut self, reload_return_mode: bool) {
        self.new_check_view.reset();

        let return_mode = self.new_check_return_mode;
        self.new_check_return_mode = ContentMode::General;
        self.mode = return_mode;
        self.error_message = None;

        if reload_return_mode {
            self.needs_reload = true;
        }
    }

    pub(super) fn navigate_back(&mut self) {
        match self.mode {
            ContentMode::NewCheck => self.close_new_check(false),
            ContentMode::SourceChecks | ContentMode::Comments => {
                self.mode = ContentMode::General;
                self.error_message = None;
            }
            ContentMode::General | ContentMode::WaitingForNextTurn => {}
        }
    }

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
                self.open_new_check(ContentMode::General, &check);
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
                self.close_new_check(false);
            }
            NewCheckAction::SaveNewRequested(check) => match self.insert_new_check(check) {
                Ok(()) => {
                    self.close_new_check(true);
                }
                Err(error) => self.error_message = Some(error),
            },
            NewCheckAction::SaveExistingRequested(check) => {
                match self.update_existing_check(check) {
                    Ok(()) => {
                        self.close_new_check(true);
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
                self.open_new_check(ContentMode::SourceChecks, &check);
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
