use super::{ContentMode, MainContentView};
use crate::database;

impl MainContentView {
    pub(super) fn is_waiting_for_next_turn(&self) -> bool {
        self.next_turn_waiting_view.is_waiting()
    }

    pub(super) fn start_next_turn_wait(&mut self) -> Result<(), String> {
        let Some(current_session) = self.current_session.as_ref() else {
            return Err("No current session is available yet.".to_string());
        };

        self.next_turn_waiting_view.start_wait(current_session);
        self.mode = ContentMode::WaitingForNextTurn;
        self.error_message = None;
        Ok(())
    }

    pub(super) fn request_new_turn(&mut self) -> Result<(), String> {
        let Some(current_session) = self.current_session.as_ref() else {
            return Err("No current session is available yet.".to_string());
        };

        if !current_session.has_new_turn() {
            let connection = database::establish_connection().map_err(|err| err.to_string())?;
            let incremented = database::current_session::increment_new_turn_number_if_needed(
                &connection,
            )
            .map_err(|err| err.to_string())?;

            if incremented {
                if let Some(current_session) = self.current_session.as_mut() {
                    current_session.new_turn_number += 1;
                }
            }
        }

        self.start_next_turn_wait()
    }

    pub(super) fn try_finish_next_turn_wait(&mut self) {
        if self
            .next_turn_waiting_view
            .try_finish_wait(self.current_session.as_ref())
        {
            self.mode = ContentMode::General;
        }
    }
}
