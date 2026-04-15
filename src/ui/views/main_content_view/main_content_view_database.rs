use super::{ContentMode, MainContentView};
use crate::database;
use crate::models::{
    check_source_type::CheckSourceType, Check, Comment, CommentType, CurrentSession, Tag,
};
use crate::ui::ui_helpers::{
    apply_check_status_update, apply_comment_content_update, find_comment_by_type_mut,
};

impl MainContentView {
    pub(super) fn sync_external_content_updates(&mut self) {
        match self.content_refresh_rx.has_changed() {
            Ok(true) => {
                self.content_refresh_rx.borrow_and_update();
                self.needs_reload = true;
            }
            Ok(false) | Err(_) => {}
        }
    }

    pub(super) fn reload_checks_if_needed(&mut self) {
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
            Ok((checks, tags, comments, source_checks, current_session)) => {
                self.checks = checks;
                self.tags = tags;
                self.comments = comments;
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
    ) -> Result<
        (
            Vec<Check>,
            Vec<Tag>,
            Vec<Comment>,
            Vec<Check>,
            Option<CurrentSession>,
        ),
        String,
    > {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        let checks = database::checks::fetch_all(&connection).map_err(|err| err.to_string())?;
        let tags = database::tags::fetch_all(&connection).map_err(|err| err.to_string())?;
        let comments = database::comments::fetch_all(&connection).map_err(|err| err.to_string())?;
        let source_checks = match source_filter {
            Some(source) => database::checks::fetch_by_source(&connection, source)
                .map_err(|err| err.to_string())?,
            None => Vec::new(),
        };
        let current_session =
            database::current_session::fetch(&connection).map_err(|err| err.to_string())?;
        Ok((checks, tags, comments, source_checks, current_session))
    }

    pub(super) fn update_check_status(
        &mut self,
        mut check: Check,
        is_checked: bool,
    ) -> Result<(), String> {
        check = apply_check_status_update(check, is_checked);
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::update(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    pub(super) fn update_comment_content(
        &mut self,
        comment_type: CommentType,
        content: String,
    ) -> Result<(), String> {
        let updated_comment = {
            let comment = find_comment_by_type_mut(&mut self.comments, comment_type.clone())
                .ok_or_else(|| {
                    self.i18n.tr(
                        "content-missing-comment-slot",
                        &[(
                            "comment_type",
                            comment_type_label(&self.i18n, &comment_type).into(),
                        )],
                    )
                })?;
            let updated = apply_comment_content_update(comment.clone(), content);
            *comment = updated.clone();
            updated
        };

        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        let id = database::comments::upsert(&connection, &updated_comment)
            .map_err(|err| err.to_string())?;

        if let Some(comment) = find_comment_by_type_mut(&mut self.comments, comment_type) {
            comment.id = id;
        }

        Ok(())
    }

    pub(super) fn insert_new_check(&mut self, check: Check) -> Result<(), String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::insert(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    pub(super) fn update_existing_check(&mut self, check: Check) -> Result<(), String> {
        let connection = database::establish_connection().map_err(|err| err.to_string())?;
        database::checks::update(&connection, &check).map_err(|err| err.to_string())?;
        self.needs_reload = true;
        self.reload_checks_if_needed();
        Ok(())
    }

    pub(super) fn count_unsent_records(&self) -> Result<usize, String> {
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
}

fn comment_type_label(i18n: &crate::i18n::I18n, comment_type: &CommentType) -> String {
    match comment_type {
        CommentType::Game => i18n.t("comment-type-game"),
        CommentType::Turn => i18n.t("comment-type-turn"),
    }
}
