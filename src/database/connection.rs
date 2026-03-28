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
    let path = database_path();
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
            text_color TEXT NOT NULL
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
            comment_type TEXT NOT NULL,
            content TEXT NOT NULL,
            is_sent INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_tags_uuid ON tags(uuid);
        CREATE INDEX IF NOT EXISTS idx_checks_uuid ON checks(uuid);
        CREATE INDEX IF NOT EXISTS idx_comments_type ON comments(comment_type);
        ",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::establish_in_memory_connection;

    #[test]
    fn comments_schema_contains_is_sent() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let mut statement = connection.prepare("PRAGMA table_info(comments)")?;
        let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
        let columns = columns.collect::<rusqlite::Result<Vec<_>>>()?;

        assert!(columns.iter().any(|column| column == "is_sent"));
        Ok(())
    }
}
