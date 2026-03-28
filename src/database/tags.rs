use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::Tag;

pub fn insert(connection: &Connection, tag: &Tag) -> Result<i64> {
    connection.execute(
        "INSERT INTO tags (uuid, name, color, text_color) VALUES (?1, ?2, ?3, ?4)",
        params![tag.uuid.to_string(), tag.name, tag.color, tag.text_color],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Tag>> {
    let mut statement =
        connection.prepare("SELECT id, uuid, name, color, text_color FROM tags ORDER BY name")?;
    let rows = statement.query_map([], row_to_tag)?;

    let tags = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(tags)
}

pub fn fetch_by_uuid(connection: &Connection, uuid: &uuid::Uuid) -> Result<Option<Tag>> {
    let mut statement =
        connection.prepare("SELECT id, uuid, name, color, text_color FROM tags WHERE uuid = ?1")?;

    let tag = statement
        .query_row([uuid.to_string()], row_to_tag)
        .optional()?;
    Ok(tag)
}

pub fn update(connection: &Connection, tag: &Tag) -> Result<()> {
    connection.execute(
        "UPDATE tags SET name = ?1, color = ?2, text_color = ?3 WHERE id = ?4",
        params![tag.name, tag.color, tag.text_color, tag.id],
    )?;

    Ok(())
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        uuid: parse_uuid(row.get(1)?),
        name: row.get(2)?,
        color: row.get(3)?,
        text_color: row.get(4)?,
    })
}

fn parse_uuid(value: String) -> uuid::Uuid {
    uuid::Uuid::parse_str(&value).unwrap_or_else(|_| uuid::Uuid::nil())
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
