use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::common::{
    bool_to_sqlite, delete_sent_missing_uuids as delete_sent_missing_uuids_in_table,
    mark_sent_by_uuids as mark_sent_by_uuids_in_table, parse_uuid, sqlite_to_bool,
};
use crate::models::Tag;

pub fn insert(connection: &Connection, tag: &Tag) -> Result<i64> {
    connection.execute(
        "INSERT INTO tags (uuid, name, color, text_color, is_sent) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            tag.uuid.to_string(),
            tag.name,
            tag.color,
            tag.text_color,
            bool_to_sqlite(tag.is_sent)
        ],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Tag>> {
    let mut statement = connection
        .prepare("SELECT id, uuid, name, color, text_color, is_sent FROM tags ORDER BY name")?;
    let rows = statement.query_map([], row_to_tag)?;

    let tags = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(tags)
}

pub fn fetch_unsent(connection: &Connection) -> Result<Vec<Tag>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, name, color, text_color, is_sent
        FROM tags
        WHERE is_sent = 0
        ORDER BY name",
    )?;
    let rows = statement.query_map([], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn fetch_by_uuid(connection: &Connection, uuid: &uuid::Uuid) -> Result<Option<Tag>> {
    let mut statement = connection
        .prepare("SELECT id, uuid, name, color, text_color, is_sent FROM tags WHERE uuid = ?1")?;

    let tag = statement
        .query_row([uuid.to_string()], row_to_tag)
        .optional()?;
    Ok(tag)
}

pub fn upsert(connection: &Connection, tag: &Tag) -> Result<i64> {
    if let Some(existing) = fetch_by_uuid(connection, &tag.uuid)? {
        connection.execute(
            "UPDATE tags SET name = ?1, color = ?2, text_color = ?3, is_sent = ?4 WHERE uuid = ?5",
            params![
                tag.name,
                tag.color,
                tag.text_color,
                bool_to_sqlite(tag.is_sent),
                tag.uuid.to_string(),
            ],
        )?;
        Ok(existing.id)
    } else {
        insert(connection, tag)
    }
}

#[cfg(test)]
pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    delete_sent_missing_uuids_in_table(connection, "tags", uuids)
}

pub fn mark_sent_by_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    mark_sent_by_uuids_in_table(connection, "tags", uuids)
}

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        uuid: parse_uuid(row.get(1)?),
        name: row.get(2)?,
        color: row.get(3)?,
        text_color: row.get(4)?,
        is_sent: sqlite_to_bool(row.get(5)?),
    })
}

#[cfg(test)]
#[path = "tags_tests.rs"]
mod tests;
