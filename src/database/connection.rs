use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::Connection;

use crate::database::checks;
use crate::database::tags;
use crate::models::{check_source_type::CheckSourceType, Check, Tag};

const INSERT_DEBUG_UNSENT_CHECK_ON_CREATE: bool = false;
const INSERT_DEBUG_TAGS_ON_START: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseStartupState {
    Ready,
    NeedsUserDecision { unsent_records: usize },
}

pub fn database_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("turn_checker_comp.db")
}

pub fn establish_connection() -> Result<Connection> {
    let path = database_path();
    establish_connection_at(&path)
}

pub fn establish_connection_at(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let is_new_database = !path.exists();
    let connection = Connection::open(path)?;
    configure_connection(&connection)?;
    maybe_insert_debug_tags(&connection)?;
    maybe_insert_debug_unsent_check(&connection, is_new_database)?;
    Ok(connection)
}

#[cfg(test)]
pub fn establish_in_memory_connection() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    configure_connection(&connection)?;
    Ok(connection)
}

fn wal_path(path: &std::path::Path) -> PathBuf {
    PathBuf::from(format!("{}-wal", path.display()))
}

fn shm_path(path: &std::path::Path) -> PathBuf {
    PathBuf::from(format!("{}-shm", path.display()))
}

fn remove_if_exists(path: &std::path::Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn reset_database() -> Result<()> {
    let path = database_path();
    reset_database_at(&path)
}

pub fn reset_database_at(path: &Path) -> Result<()> {
    remove_if_exists(path)?;
    remove_if_exists(&wal_path(path))?;
    remove_if_exists(&shm_path(path))?;

    let _connection = establish_connection_at(path)?;
    Ok(())
}

fn configure_connection(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            text_color TEXT NOT NULL,
            is_sent INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS checks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            detail TEXT,
            source TEXT NOT NULL,
            repeat_type TEXT NOT NULL,
            repeat_value INTEGER,
            tag_uuid TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            is_mandatory INTEGER NOT NULL DEFAULT 0,
            is_checked INTEGER NOT NULL DEFAULT 0,
            is_sent INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL UNIQUE,
            comment_type TEXT NOT NULL,
            content TEXT NOT NULL,
            is_sent INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS current_session (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            game_uuid TEXT,
            game_name TEXT NOT NULL,
            turn_number INTEGER NOT NULL,
            new_turn_number INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tags_uuid ON tags(uuid);
        CREATE INDEX IF NOT EXISTS idx_checks_uuid ON checks(uuid);
        CREATE INDEX IF NOT EXISTS idx_comments_uuid ON comments(uuid);
        CREATE INDEX IF NOT EXISTS idx_comments_type ON comments(comment_type);
        ",
    )?;

    Ok(())
}

pub fn inspect_startup_state() -> Result<DatabaseStartupState> {
    let path = database_path();
    inspect_startup_state_at(&path)
}

pub fn inspect_startup_state_at(path: &Path) -> Result<DatabaseStartupState> {
    let file_exists = path.exists();
    let connection = establish_connection_at(path)?;
    if !file_exists {
        return Ok(DatabaseStartupState::Ready);
    }

    let unsent_records = count_unsent_records(&connection)?;
    if unsent_records > 0 {
        Ok(DatabaseStartupState::NeedsUserDecision { unsent_records })
    } else {
        drop(connection);
        reset_database_at(path)?;
        Ok(DatabaseStartupState::Ready)
    }
}

fn count_unsent_records(connection: &Connection) -> Result<usize> {
    let checks: i64 =
        connection.query_row("SELECT COUNT(*) FROM checks WHERE is_sent = 0", [], |row| {
            row.get(0)
        })?;
    let comments: i64 = connection.query_row(
        "SELECT COUNT(*) FROM comments WHERE is_sent = 0",
        [],
        |row| row.get(0),
    )?;
    let tags: i64 =
        connection.query_row("SELECT COUNT(*) FROM tags WHERE is_sent = 0", [], |row| {
            row.get(0)
        })?;

    Ok((checks + comments + tags) as usize)
}

fn maybe_insert_debug_unsent_check(connection: &Connection, is_new_database: bool) -> Result<()> {
    if INSERT_DEBUG_UNSENT_CHECK_ON_CREATE && is_new_database {
        insert_debug_unsent_check(connection)?;
    }

    Ok(())
}

fn maybe_insert_debug_tags(connection: &Connection) -> Result<()> {
    if INSERT_DEBUG_TAGS_ON_START {
        insert_debug_tags(connection)?;
    }

    Ok(())
}

fn insert_debug_tags(connection: &Connection) -> Result<()> {
    for (name, color, text_color) in [
        ("war", "#C0392B", "#FFFFFF"),
        ("economy", "#27AE60", "#FFFFFF"),
        ("diplomacy", "#2980B9", "#FFFFFF"),
    ] {
        let tag = Tag::new(name, color, text_color);
        tags::insert(connection, &tag)?;
    }

    Ok(())
}

fn insert_debug_unsent_check(connection: &Connection) -> Result<()> {
    let mut check = Check::new("Debug unsent check");
    check.detail = Some("Inserted automatically for startup debug flows.".to_string());
    check.source = CheckSourceType::Turn;
    check.is_sent = false;
    checks::insert(connection, &check)?;
    Ok(())
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
