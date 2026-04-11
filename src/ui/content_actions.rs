use super::checklist::ChecklistAction;
use super::new_check::NewCheckAction;
use super::{ContentAction, ContentMode, MainContentView};

impl MainContentView {
    pub(super) fn handle_restart_click(&mut self) -> Option<ContentAction> {
        match self.count_unsent_records() {
            Ok(0) => Some(ContentAction::RestartRequested),
            Ok(unsent_records) => {
                self.restart_confirmation_unsent_checks = Some(unsent_records);
                None
            }
            Err(error) => {
                self.error_message = Some(error);
                None
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
        }
    }

    pub(super) fn handle_new_check_action(&mut self, action: NewCheckAction) {
        match action {
            NewCheckAction::Cancelled => {
                self.mode = ContentMode::General;
                self.error_message = None;
            }
            NewCheckAction::SaveRequested(check) => match self.insert_new_check(check) {
                Ok(()) => {
                    self.new_check_view.reset();
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
                Err(error) => self.error_message = Some(error),
            },
            NewCheckAction::ValidationFailed(error) => {
                self.error_message = Some(error);
            }
        }
    }
}
