use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;

pub fn database_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("turn_checker_comp.db")
}

pub fn establish_connection() -> Result<Connection> {
    establish_connection_at(database_path())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseStartupState {
    Ready,
    NeedsUserDecision { unsent_records: usize },
}

pub fn establish_connection_at(path: PathBuf) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let connection = Connection::open(path)?;
    configure_connection(&connection)?;
    Ok(connection)
}

pub fn establish_in_memory_connection() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    configure_connection(&connection)?;
    Ok(connection)
}

pub fn inspect_startup_state() -> Result<DatabaseStartupState> {
    inspect_startup_state_at(database_path())
}

pub fn inspect_startup_state_at(path: PathBuf) -> Result<DatabaseStartupState> {
    let file_exists = path.exists();
    let connection = establish_connection_at(path)?;
    if !file_exists {
        return Ok(DatabaseStartupState::Ready);
    }

    let unsent_records = count_unsent_records(&connection)?;
    if unsent_records > 0 {
        Ok(DatabaseStartupState::NeedsUserDecision { unsent_records })
    } else {
        Ok(DatabaseStartupState::Ready)
    }
}

pub fn reset_database() -> Result<()> {
    reset_database_at(database_path())
}

pub fn reset_database_at(path: PathBuf) -> Result<()> {
    remove_if_exists(&path)?;
    remove_if_exists(&wal_path(&path))?;
    remove_if_exists(&shm_path(&path))?;

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
            repeat_type TEXT NOT NULL,
            repeat_value INTEGER,
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

        CREATE INDEX IF NOT EXISTS idx_tags_uuid ON tags(uuid);
        CREATE INDEX IF NOT EXISTS idx_checks_uuid ON checks(uuid);
        CREATE INDEX IF NOT EXISTS idx_comments_uuid ON comments(uuid);
        CREATE INDEX IF NOT EXISTS idx_comments_type ON comments(comment_type);
        ",
    )?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{
        establish_connection_at, inspect_startup_state_at, reset_database_at, DatabaseStartupState,
    };
    use crate::database::checks;
    use crate::models::Check;

    #[test]
    fn startup_state_detects_unsent_records() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("turn-checker-db-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("startup.db");

        assert_eq!(
            inspect_startup_state_at(db_path.clone())?,
            DatabaseStartupState::Ready
        );

        let connection = establish_connection_at(db_path.clone())?;
        let check = Check::new("Scout");
        checks::insert(&connection, &check)?;

        assert_eq!(
            inspect_startup_state_at(db_path)?,
            DatabaseStartupState::NeedsUserDecision { unsent_records: 1 }
        );

        Ok(())
    }

    #[test]
    fn reset_database_recreates_empty_schema() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("turn-checker-reset-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        let db_path = temp_dir.join("reset.db");

        let connection = establish_connection_at(db_path.clone())?;
        let check = Check::new("Cleanup");
        checks::insert(&connection, &check)?;
        drop(connection);

        reset_database_at(db_path.clone())?;

        let connection = establish_connection_at(db_path)?;
        assert!(checks::fetch_all(&connection)?.is_empty());

        Ok(())
    }
}
