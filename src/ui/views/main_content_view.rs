mod main_content_view_actions;
mod main_content_view_database;
mod main_content_view_dialogs;
mod main_content_view_next_turn;
mod main_content_view_toolbar;

use crate::i18n::I18n;
use crate::models::{check_source_type::CheckSourceType, Check, Comment, CurrentSession, Tag};
use crate::ui::theme::Theme;
use eframe::egui;
use tokio::sync::watch;

use super::checklist_view::ChecklistView;
use super::comments_view::CommentsView;
use super::new_check_view::NewCheckView;
use super::next_turn_waiting_view::NextTurnWaitingView;
use super::source_checks_view::SourceChecksView;

pub struct MainContentView {
    i18n: I18n,
    mode: ContentMode,
    checks: Vec<Check>,
    source_checks: Vec<Check>,
    tags: Vec<Tag>,
    comments: Vec<Comment>,
    current_session: Option<CurrentSession>,
    checklist_view: ChecklistView,
    comments_view: CommentsView,
    new_check_view: NewCheckView,
    next_turn_waiting_view: NextTurnWaitingView,
    source_checks_view: SourceChecksView,
    source_checks_config: Option<SourceChecksConfig>,
    new_check_return_mode: ContentMode,
    error_message: Option<String>,
    new_turn_confirmation_open: Option<usize>,
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
    title_key: &'static str,
    source: CheckSourceType,
}

impl MainContentView {
    pub fn new(content_refresh_rx: watch::Receiver<u64>, i18n: I18n) -> Self {
        Self {
            i18n,
            mode: ContentMode::General,
            checks: Vec::new(),
            source_checks: Vec::new(),
            tags: Vec::new(),
            comments: Vec::new(),
            current_session: None,
            checklist_view: ChecklistView::default(),
            comments_view: CommentsView::default(),
            new_check_view: NewCheckView::default(),
            next_turn_waiting_view: NextTurnWaitingView::default(),
            source_checks_view: SourceChecksView::default(),
            source_checks_config: None,
            new_check_return_mode: ContentMode::General,
            error_message: None,
            new_turn_confirmation_open: None,
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
        self.comments.clear();
        self.current_session = None;
        self.checklist_view = ChecklistView::default();
        self.comments_view = CommentsView::default();
        self.new_check_view.reset();
        self.next_turn_waiting_view = NextTurnWaitingView::default();
        self.source_checks_view = SourceChecksView::default();
        self.source_checks_config = None;
        self.new_check_return_mode = ContentMode::General;
        self.error_message = None;
        self.new_turn_confirmation_open = None;
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
mod main_content_view_tests;
