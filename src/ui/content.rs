mod check_cards;
mod checklist;
mod comments;
#[path = "content_actions.rs"]
mod content_actions;
#[path = "content_database.rs"]
mod database_ops;
#[path = "content_dialogs.rs"]
mod dialogs;
#[path = "content_helpers.rs"]
mod helpers;
mod new_check;
mod new_check_draft;
#[path = "content_next_turn.rs"]
mod next_turn;
mod next_turn_waiting;
mod source_checks;
mod toggle_button;
#[path = "content_toolbar.rs"]
mod toolbar;

use crate::models::{check_source_type::CheckSourceType, Check, CurrentSession, Tag};
use crate::ui::theme::Theme;
use eframe::egui;
use tokio::sync::watch;

use self::checklist::ChecklistView;
use self::comments::CommentsView;
use self::new_check::NewCheckView;
use self::next_turn_waiting::NextTurnWaitingView;
use self::source_checks::SourceChecksView;

pub struct MainContentView {
    mode: ContentMode,
    checks: Vec<Check>,
    source_checks: Vec<Check>,
    tags: Vec<Tag>,
    current_session: Option<CurrentSession>,
    checklist_view: ChecklistView,
    comments_view: CommentsView,
    new_check_view: NewCheckView,
    next_turn_waiting_view: NextTurnWaitingView,
    source_checks_view: SourceChecksView,
    source_checks_config: Option<SourceChecksConfig>,
    error_message: Option<String>,
    restart_confirmation_unsent_checks: Option<usize>,
    needs_reload: bool,
    content_refresh_rx: watch::Receiver<u64>,
}

pub enum ContentAction {
    NewTurnNotifRequested,
    RestartRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    General,
    WaitingForNextTurn,
    NewCheck,
    SourceChecks,
    Comments,
}

#[derive(Clone)]
struct SourceChecksConfig {
    title: &'static str,
    source: CheckSourceType,
}

impl MainContentView {
    pub fn new(content_refresh_rx: watch::Receiver<u64>) -> Self {
        Self {
            mode: ContentMode::General,
            checks: Vec::new(),
            source_checks: Vec::new(),
            tags: Vec::new(),
            current_session: None,
            checklist_view: ChecklistView::default(),
            comments_view: CommentsView::default(),
            new_check_view: NewCheckView::default(),
            next_turn_waiting_view: NextTurnWaitingView::default(),
            source_checks_view: SourceChecksView::default(),
            source_checks_config: None,
            error_message: None,
            restart_confirmation_unsent_checks: None,
            needs_reload: true,
            content_refresh_rx,
        }
    }

    pub fn set_error_message(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    pub fn cancel_next_turn_wait(&mut self, message: impl Into<String>) {
        self.next_turn_waiting_view.cancel_wait();
        self.mode = ContentMode::General;
        self.error_message = Some(message.into());
    }

    pub fn prepare_for_restart(&mut self) {
        self.mode = ContentMode::General;
        self.checks.clear();
        self.source_checks.clear();
        self.tags.clear();
        self.current_session = None;
        self.checklist_view = ChecklistView::default();
        self.comments_view = CommentsView::default();
        self.new_check_view.reset();
        self.next_turn_waiting_view = NextTurnWaitingView::default();
        self.source_checks_view = SourceChecksView::default();
        self.source_checks_config = None;
        self.error_message = None;
        self.restart_confirmation_unsent_checks = None;
        self.needs_reload = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<ContentAction> {
        let theme = Theme::from_visuals(ui.visuals());
        let mut action = None;
        self.sync_external_content_updates();
        self.reload_checks_if_needed();
        self.show_root_frame(ui, &theme, &mut action);
        action
    }
}

#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
