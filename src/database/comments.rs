use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::common::{
    bool_to_sqlite, delete_sent_missing_uuids as delete_sent_missing_uuids_in_table,
    mark_sent_by_uuids as mark_sent_by_uuids_in_table, parse_uuid, sqlite_to_bool,
};
use crate::models::{Comment, CommentType};

pub fn insert(connection: &Connection, comment: &Comment) -> Result<i64> {
    connection.execute(
        "INSERT INTO comments (uuid, comment_type, content, is_sent) VALUES (?1, ?2, ?3, ?4)",
        params![
            comment.uuid.to_string(),
            comment.comment_type.as_str(),
            comment.content,
            bool_to_sqlite(comment.is_sent)
        ],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Comment>> {
    let mut statement = connection
        .prepare("SELECT id, uuid, comment_type, content, is_sent FROM comments ORDER BY id")?;
    let rows = statement.query_map([], row_to_comment)?;

    let comments = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(comments)
}

pub fn fetch_unsent(connection: &Connection) -> Result<Vec<Comment>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, comment_type, content, is_sent
        FROM comments
        WHERE is_sent = 0
        ORDER BY id",
    )?;
    let rows = statement.query_map([], row_to_comment)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn fetch_by_uuid(connection: &Connection, uuid: &uuid::Uuid) -> Result<Option<Comment>> {
    let mut statement = connection
        .prepare("SELECT id, uuid, comment_type, content, is_sent FROM comments WHERE uuid = ?1")?;

    let comment = statement
        .query_row([uuid.to_string()], row_to_comment)
        .optional()?;
    Ok(comment)
}

pub fn upsert(connection: &Connection, comment: &Comment) -> Result<i64> {
    if let Some(existing) = fetch_by_uuid(connection, &comment.uuid)? {
        connection.execute(
            "UPDATE comments SET comment_type = ?1, content = ?2, is_sent = ?3 WHERE uuid = ?4",
            params![
                comment.comment_type.as_str(),
                comment.content,
                bool_to_sqlite(comment.is_sent),
                comment.uuid.to_string()
            ],
        )?;
        Ok(existing.id)
    } else {
        insert(connection, comment)
    }
}

#[cfg(test)]
pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM comments WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    delete_sent_missing_uuids_in_table(connection, "comments", uuids)
}

pub fn mark_sent_by_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    mark_sent_by_uuids_in_table(connection, "comments", uuids)
}

fn row_to_comment(row: &Row<'_>) -> rusqlite::Result<Comment> {
    let raw_type: String = row.get(2)?;
    Ok(Comment {
        id: row.get(0)?,
        uuid: parse_uuid(row.get(1)?),
        comment_type: CommentType::from_str(&raw_type),
        content: row.get(3)?,
        is_sent: sqlite_to_bool(row.get(4)?),
    })
}

#[cfg(test)]
#[path = "comments_tests.rs"]
mod tests;
