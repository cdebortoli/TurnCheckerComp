use eframe::egui::{self, RichText};
use uuid::Uuid;

use crate::i18n::I18n;
use crate::models::CurrentSession;
use crate::ui::theme::Theme;

#[derive(Default)]
pub(super) struct NextTurnWaitingView {
    wait_state: Option<NextTurnWaitState>,
}

#[derive(Clone, Copy)]
struct NextTurnWaitState {
    baseline_turn_number: i32,
    game_uuid: Option<Uuid>,
}

impl NextTurnWaitingView {
    pub(super) fn is_waiting(&self) -> bool {
        self.wait_state.is_some()
    }

    pub(super) fn start_wait(&mut self, current_session: &CurrentSession) {
        self.wait_state = Some(NextTurnWaitState {
            baseline_turn_number: current_session.turn_number,
            game_uuid: current_session.game_uuid,
        });
    }

    pub(super) fn cancel_wait(&mut self) {
        self.wait_state = None;
    }

    pub(super) fn try_finish_wait(&mut self, current_session: Option<&CurrentSession>) -> bool {
        let Some(wait_state) = self.wait_state else {
            return false;
        };

        let Some(current_session) = current_session else {
            return false;
        };

        if let Some(expected_game_uuid) = wait_state.game_uuid {
            if current_session.game_uuid != Some(expected_game_uuid) {
                return false;
            }
        }

        if current_session.turn_number > wait_state.baseline_turn_number {
            self.wait_state = None;
            return true;
        }

        false
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, i18n: &I18n) {
        egui::Frame::new()
            .fill(theme.bg_turn_card)
            .inner_margin(theme.card_padding)
            .corner_radius(theme.corner_radius)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(
                        RichText::new(i18n.t("waiting-next-turn-title")).color(theme.text_primary),
                    );
                    ui.add_space(theme.spacing_md);
                    ui.spinner();
                    ui.add_space(theme.spacing_md);
                    ui.label(
                        RichText::new(i18n.t("waiting-next-turn-description"))
                            .color(theme.text_secondary),
                    );
                });
            });
    }
}
