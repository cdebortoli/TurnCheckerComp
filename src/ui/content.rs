mod check_cards;
mod checklist;
mod comments;
mod new_check;
mod new_check_draft;
mod next_turn_waiting;
mod source_checks;
mod toggle_button;

use crate::database;
use crate::models::{check_source_type::CheckSourceType, Check, CurrentSession, Tag};
use crate::ui::theme::Theme;
use eframe::egui::{self, RichText};
use tokio::sync::watch;
use uuid::Uuid;

use self::checklist::{ChecklistAction, ChecklistView};
use self::comments::CommentsView;
use self::new_check::{NewCheckAction, NewCheckView};
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

    fn is_waiting_for_next_turn(&self) -> bool {
        self.next_turn_waiting_view.is_waiting()
    }

    fn start_next_turn_wait(&mut self) -> Result<(), String> {
        let Some(current_session) = self.current_session.as_ref() else {
            return Err("No current session is available yet.".to_string());
        };

        self.next_turn_waiting_view.start_wait(current_session);
        self.mode = ContentMode::WaitingForNextTurn;
        self.error_message = None;
        Ok(())
    }

    fn try_finish_next_turn_wait(&mut self) {
        if self
            .next_turn_waiting_view
            .try_finish_wait(self.current_session.as_ref())
        {
            self.mode = ContentMode::General;
        }
    }

    fn sync_external_content_updates(&mut self) {
        match self.content_refresh_rx.has_changed() {
            Ok(true) => {
                self.content_refresh_rx.borrow_and_update();
                self.needs_reload = true;
            }
            Ok(false) | Err(_) => {}
        }
    }

    fn reload_checks_if_needed(&mut self) {
        if !self.needs_reload {
            return;
        }

        let source_filter = match self.mode {
            ContentMode::SourceChecks => self
                .source_checks_config
                .as_ref()
                .map(|config| config.source.clone()),
            _ => None,
        };

        match Self::load_content(source_filter) {
            Ok((checks, tags, source_checks, current_session)) => {
                self.checks = checks;
                self.tags = tags;
                self.source_checks = source_checks;
                self.current_session = current_session;
                self.try_finish_next_turn_wait();
                self.error_message = None;
            }
            Err(error) => self.error_message = Some(error),
        }
        self.needs_reload = false;
    }

    fn load_content(
        source_filter: Option<CheckSourceType>,
    ) -> Result<(Vec<Check>, Vec<Tag>, Vec<Check>, Option<CurrentSession>), String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        let checks = database::checks::fetch_all(&connection).map_err(|err| err.to_string())?;
        let tags = database::tags::fetch_all(&connection).map_err(|err| err.to_string())?;
        let source_checks = match source_filter {
            Some(source) => database::checks::fetch_by_source(&connection, source)
                .map_err(|err| err.to_string())?,
            None => Vec::new(),
        };
        let current_session =
            database::current_session::fetch(&connection).map_err(|err| err.to_string())?;
        Ok((checks, tags, source_checks, current_session))
    }

    fn update_check_status(&mut self, mut check: Check, is_checked: bool) -> Result<(), String> {
        check = apply_check_status_update(check, is_checked);
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::update(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    fn insert_new_check(&mut self, check: Check) -> Result<(), String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::insert(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    fn handle_restart_click(&mut self) -> Option<ContentAction> {
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

    fn count_unsent_records(&self) -> Result<usize, String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        let checks = database::checks::count_unsent(&connection).map_err(|err| err.to_string())?;
        let comments = database::comments::fetch_unsent(&connection)
            .map_err(|err| err.to_string())?
            .len();
        let tags = database::tags::fetch_unsent(&connection)
            .map_err(|err| err.to_string())?
            .len();
        Ok(checks + comments + tags)
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<ContentAction> {
        let theme = Theme::from_visuals(ui.visuals());
        let mut action = None;
        self.sync_external_content_updates();
        self.reload_checks_if_needed();
        self.show_root_frame(ui, &theme, &mut action);
        action
    }

    fn show_root_frame(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        egui::Frame::new()
            .fill(theme.bg_primary)
            .inner_margin(theme.spacing_lg)
            .show(ui, |ui| {
                self.show_top_bar(ui, theme, action);
                ui.add_space(theme.spacing_md);
                self.show_error_message(ui, theme);
                self.show_active_content(ui, theme);
            });
        self.show_restart_confirmation(ui, theme, action);
    }

    fn show_top_bar(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        ui.horizontal(|ui| match self.mode {
            ContentMode::General | ContentMode::WaitingForNextTurn => {
                ui.add_enabled_ui(!self.is_waiting_for_next_turn(), |ui| {
                    self.show_next_turn_button(ui, theme, action);
                    self.show_mode_button(ui, theme, "New Check", ContentMode::NewCheck);
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Game's turns checks",
                        CheckSourceType::Game,
                    );
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Game's checks",
                        CheckSourceType::GlobalGame,
                    );
                    self.show_source_checks_button(
                        ui,
                        theme,
                        "Template's checks",
                        CheckSourceType::Blueprint,
                    );
                    self.show_mode_button(ui, theme, "Comments", ContentMode::Comments);
                    self.show_restart_button(ui, theme, action);
                });
            }
            _ => {
                if ui
                    .button(RichText::new("Back").color(theme.text_primary))
                    .clicked()
                {
                    self.mode = ContentMode::General;
                    self.error_message = None;
                }
            }
        });
    }

    fn show_next_turn_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let button = egui::Button::new(RichText::new("Next turn").color(theme.text_primary))
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            match self.start_next_turn_wait() {
                Ok(()) => *action = Some(ContentAction::NewTurnNotifRequested),
                Err(error) => self.error_message = Some(error),
            }
        }
    }

    fn show_mode_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &str,
        target_mode: ContentMode,
    ) {
        let button = egui::Button::new(RichText::new(label).color(theme.text_primary))
            .fill(if self.mode == target_mode {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.mode = target_mode;
            self.error_message = None;
        }
    }

    fn show_source_checks_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &'static str,
        source: CheckSourceType,
    ) {
        let is_active = self.mode == ContentMode::SourceChecks
            && self
                .source_checks_config
                .as_ref()
                .is_some_and(|config| config.title == label && config.source == source);
        let button = egui::Button::new(RichText::new(label).color(theme.text_primary))
            .fill(if is_active {
                theme.accent
            } else {
                theme.bg_secondary
            })
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            self.mode = ContentMode::SourceChecks;
            self.source_checks_config = Some(SourceChecksConfig {
                title: label,
                source,
            });
            self.error_message = None;
            self.needs_reload = true;
        }
    }

    fn show_restart_button(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let button = egui::Button::new(RichText::new("Restart").color(theme.text_primary))
            .fill(theme.bg_secondary)
            .corner_radius(theme.corner_radius);

        if ui.add(button).clicked() {
            *action = self.handle_restart_click();
        }
    }

    fn show_restart_confirmation(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        action: &mut Option<ContentAction>,
    ) {
        let Some(unsent_checks) = self.restart_confirmation_unsent_checks else {
            return;
        };

        let ctx = ui.ctx().clone();
        egui::Window::new("Restart")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "The database contains {unsent_checks} unsent check(s)."
                    ))
                    .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(
                        "Restarting will delete and recreate the database, then return to the pairing screen.",
                    )
                    .color(theme.text_muted),
                );
                ui.add_space(theme.spacing_md);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.restart_confirmation_unsent_checks = None;
                    }

                    if ui.button("Restart").clicked() {
                        self.restart_confirmation_unsent_checks = None;
                        *action = Some(ContentAction::RestartRequested);
                    }
                });
            });
    }

    fn show_error_message(&self, ui: &mut egui::Ui, theme: &Theme) {
        if let Some(error) = &self.error_message {
            ui.label(RichText::new(error).color(theme.destructive));
            ui.add_space(theme.spacing_md);
        }
    }

    fn show_active_content(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        match self.mode {
            ContentMode::General => {
                let action = self.checklist_view.show(
                    ui,
                    theme,
                    self.current_session.as_ref(),
                    &self.checks,
                    &self.tags,
                );
                if let Some(action) = action {
                    self.handle_checklist_action(action);
                }
            }
            ContentMode::WaitingForNextTurn => {
                self.next_turn_waiting_view.show(ui, theme);
            }
            ContentMode::NewCheck => {
                let action = self.new_check_view.show(ui, theme, &self.tags);
                if let Some(action) = action {
                    self.handle_new_check_action(action);
                }
            }
            ContentMode::SourceChecks => {
                if let Some(config) = self.source_checks_config.as_ref() {
                    self.source_checks_view.show(
                        ui,
                        theme,
                        config.title,
                        &self.source_checks,
                        &self.tags,
                    );
                }
            }
            ContentMode::Comments => {
                self.comments_view.show(ui, theme);
            }
        }
    }

    fn handle_checklist_action(&mut self, action: ChecklistAction) {
        match action {
            ChecklistAction::CheckToggled { check, is_checked } => {
                if let Err(error) = self.update_check_status(check, is_checked) {
                    self.error_message = Some(error);
                }
            }
        }
    }

    fn handle_new_check_action(&mut self, action: NewCheckAction) {
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

fn apply_check_status_update(mut check: Check, is_checked: bool) -> Check {
    check.is_checked = is_checked;
    check.is_sent = false;
    check
}

fn find_tag_by_uuid<'a>(tags: &'a [Tag], tag_uuid: Option<Uuid>) -> Option<&'a Tag> {
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

fn tag_fill_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.color).unwrap_or_else(|| egui::Color32::from_rgb(99, 99, 102))
}

fn tag_text_color(tag: &Tag) -> egui::Color32 {
    parse_hex_color(&tag.text_color).unwrap_or(egui::Color32::WHITE)
}

fn show_tag_capsule(ui: &mut egui::Ui, tag: &Tag) {
    egui::Frame::new()
        .fill(tag_fill_color(tag))
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

#[cfg(test)]
mod tests {
    use super::{apply_check_status_update, ContentMode, MainContentView};
    use crate::models::{Check, CurrentSession};
    use tokio::sync::watch;
    use uuid::Uuid;

    #[test]
    fn external_refresh_marks_content_dirty() {
        let (content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
        let mut view = MainContentView::new(content_refresh_rx);
        view.needs_reload = false;

        content_refresh_tx
            .send(1)
            .expect("refresh signal should send");
        view.sync_external_content_updates();

        assert!(view.needs_reload);
        assert!(!view.content_refresh_rx.has_changed().unwrap());
    }

    #[test]
    fn local_status_update_marks_check_unsent() {
        let mut check = Check::new("Scout");
        check.is_sent = true;

        let updated = apply_check_status_update(check, true);

        assert!(updated.is_checked);
        assert!(!updated.is_sent);
    }

    #[test]
    fn next_turn_wait_unlocks_after_turn_increase_for_same_game() {
        let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
        let game_uuid = Uuid::new_v4();
        let mut view = MainContentView::new(content_refresh_rx);
        view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));

        view.start_next_turn_wait().expect("wait should start");
        assert!(view.is_waiting_for_next_turn());
        assert_eq!(view.mode, ContentMode::WaitingForNextTurn);

        view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 6));
        view.try_finish_next_turn_wait();

        assert!(!view.is_waiting_for_next_turn());
        assert_eq!(view.mode, ContentMode::General);
    }

    #[test]
    fn next_turn_wait_stays_locked_without_turn_increase() {
        let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
        let game_uuid = Uuid::new_v4();
        let mut view = MainContentView::new(content_refresh_rx);
        view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));

        view.start_next_turn_wait().expect("wait should start");
        view.current_session = Some(CurrentSession::new(Some(game_uuid), "Civ VI", 5));
        view.try_finish_next_turn_wait();

        assert!(view.is_waiting_for_next_turn());
    }

    #[test]
    fn next_turn_wait_stays_locked_for_different_game_uuid() {
        let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
        let mut view = MainContentView::new(content_refresh_rx);
        view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));

        view.start_next_turn_wait().expect("wait should start");
        view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 6));
        view.try_finish_next_turn_wait();

        assert!(view.is_waiting_for_next_turn());
    }

    #[test]
    fn cancel_next_turn_wait_clears_wait_state_and_sets_error() {
        let (_content_refresh_tx, content_refresh_rx) = watch::channel(0_u64);
        let mut view = MainContentView::new(content_refresh_rx);
        view.current_session = Some(CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 5));
        view.start_next_turn_wait().expect("wait should start");

        view.cancel_next_turn_wait("push failed");

        assert!(!view.is_waiting_for_next_turn());
        assert_eq!(view.mode, ContentMode::General);
        assert_eq!(view.error_message.as_deref(), Some("push failed"));
    }
}
