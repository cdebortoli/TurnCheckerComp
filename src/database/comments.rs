use std::collections::HashSet;

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

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

pub fn update(connection: &Connection, comment: &Comment) -> Result<()> {
    connection.execute(
        "UPDATE comments SET uuid = ?1, comment_type = ?2, content = ?3, is_sent = ?4 WHERE id = ?5",
        params![
            comment.uuid.to_string(),
            comment.comment_type.as_str(),
            comment.content,
            bool_to_sqlite(comment.is_sent),
            comment.id
        ],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM comments WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    let retained: HashSet<uuid::Uuid> = uuids.iter().copied().collect();
    let mut statement = connection.prepare("SELECT id, uuid FROM comments WHERE is_sent = 1")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, parse_uuid(row.get::<_, String>(1)?)))
    })?;

    let mut deleted = 0;
    for row in rows {
        let (id, uuid) = row?;
        if !retained.contains(&uuid) {
            delete(connection, id)?;
            deleted += 1;
        }
    }

    Ok(deleted)
}

pub fn mark_sent_by_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    let mut updated = 0;
    for uuid in uuids {
        updated += connection.execute(
            "UPDATE comments SET is_sent = 1 WHERE uuid = ?1",
            [uuid.to_string()],
        )?;
    }

    Ok(updated)
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

fn parse_uuid(value: String) -> uuid::Uuid {
    uuid::Uuid::parse_str(&value).unwrap_or_else(|_| uuid::Uuid::nil())
}

fn bool_to_sqlite(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn sqlite_to_bool(value: i64) -> bool {
    value != 0
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::database::connection::establish_in_memory_connection;
    use crate::models::{Comment, CommentType};

    #[test]
    fn comment_crud_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let mut comment = Comment::new(CommentType::Turn, "Remember the timing");

        let id = super::insert(&connection, &comment)?;
        comment.id = id;

        let fetched = super::fetch_by_uuid(&connection, &comment.uuid)?.expect("comment exists");
        assert_eq!(fetched.comment_type, CommentType::Turn);
        assert!(!fetched.is_sent);

        comment.comment_type = CommentType::Game;
        comment.content = "Whole match note".to_string();
        comment.is_sent = true;
        super::update(&connection, &comment)?;

        let comments = super::fetch_all(&connection)?;
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].comment_type, CommentType::Game);
        assert!(comments[0].is_sent);

        super::delete(&connection, id)?;
        assert!(super::fetch_all(&connection)?.is_empty());

        Ok(())
    }
}
