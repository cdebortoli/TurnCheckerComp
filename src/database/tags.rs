use std::collections::HashSet;

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

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

pub fn fetch_unsent(connection: &Connection, limit: Option<usize>) -> Result<Vec<Tag>> {
    match limit {
        Some(limit) => {
            let mut statement = connection.prepare(
                "SELECT id, uuid, name, color, text_color, is_sent
                 FROM tags
                 WHERE is_sent = 0
                 ORDER BY name
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([limit as i64], row_to_tag)?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        }
        None => {
            let mut statement = connection.prepare(
                "SELECT id, uuid, name, color, text_color, is_sent
                 FROM tags
                 WHERE is_sent = 0
                 ORDER BY name",
            )?;
            let rows = statement.query_map([], row_to_tag)?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        }
    }
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

pub fn update(connection: &Connection, tag: &Tag) -> Result<()> {
    connection.execute(
        "UPDATE tags SET name = ?1, color = ?2, text_color = ?3, is_sent = ?4 WHERE id = ?5",
        params![
            tag.name,
            tag.color,
            tag.text_color,
            bool_to_sqlite(tag.is_sent),
            tag.id
        ],
    )?;

    Ok(())
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    let retained: HashSet<uuid::Uuid> = uuids.iter().copied().collect();
    let mut statement = connection.prepare("SELECT id, uuid FROM tags WHERE is_sent = 1")?;
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
            "UPDATE tags SET is_sent = 1 WHERE uuid = ?1",
            [uuid.to_string()],
        )?;
    }

    Ok(updated)
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
    use crate::models::Tag;

    #[test]
    fn tag_crud_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let mut tag = Tag::new("Attack", "#FF0000", "#FFFFFF");

        let id = super::insert(&connection, &tag)?;
        tag.id = id;

        let fetched = super::fetch_by_uuid(&connection, &tag.uuid)?.expect("tag exists");
        assert_eq!(fetched.name, "Attack");

        tag.name = "Defense".to_string();
        super::update(&connection, &tag)?;

        let tags = super::fetch_all(&connection)?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Defense");

        super::delete(&connection, id)?;
        assert!(super::fetch_all(&connection)?.is_empty());

        Ok(())
    }
}
