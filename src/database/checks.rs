use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::common::{
    bool_to_sqlite, delete_sent_missing_uuids as delete_sent_missing_uuids_in_table,
    mark_sent_by_uuids as mark_sent_by_uuids_in_table, parse_uuid, sqlite_to_bool,
};
use crate::models::check_source_type::CheckSourceType;
use crate::models::{Check, CheckRepeatType};

pub fn insert(connection: &Connection, check: &Check) -> Result<i64> {
    // Auto-generate position as max existing position + 1
    let max_position: Option<i32> = connection
        .query_row("SELECT COALESCE(MAX(position), 0) FROM checks", [], |row| {
            row.get(0)
        })
        .optional()?;
    let position = match max_position {
        Some(max) => max + 1,
        None => 1,
    };

    let source = check.source.to_storage();
    let (repeat_type, repeat_value) = check.repeat_case.to_storage();

    connection.execute(
        "INSERT INTO checks (
            uuid, name, detail, source, repeat_type, repeat_value, tag_uuid, position, is_mandatory, is_checked, is_sent
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            check.uuid.to_string(),
            check.name,
            check.detail,
            source,
            repeat_type,
            repeat_value,
            check.tag_uuid.map(|uuid| uuid.to_string()),
            position,
            bool_to_sqlite(check.is_mandatory),
            bool_to_sqlite(check.is_checked),
            bool_to_sqlite(check.is_sent),
        ],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Check>> {
    let global_game_source = CheckSourceType::GlobalGame.to_storage();
    let turn_source = CheckSourceType::Turn.to_storage();

    let mut statement = connection.prepare(
      "SELECT id, uuid, name, detail, source, repeat_type, repeat_value, tag_uuid, position, is_mandatory, is_checked, is_sent
      FROM checks
      WHERE source IN (?1, ?2)
      ORDER BY
        CASE
          WHEN source = ?1 THEN 0
          ELSE 1
        END,
        position,
        name",
    )?;
    let rows = statement.query_map([global_game_source, turn_source], row_to_check)?;

    let checks = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(checks)
}

pub fn fetch_by_source(connection: &Connection, source: CheckSourceType) -> Result<Vec<Check>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, name, detail, source, repeat_type, repeat_value, tag_uuid, position, is_mandatory, is_checked, is_sent
         FROM checks
         WHERE source = ?1
         ORDER BY position, name",
    )?;
    let rows = statement.query_map([source.to_storage()], row_to_check)?;

    let checks = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(checks)
}

pub fn fetch_unsent(connection: &Connection) -> Result<Vec<Check>> {
    let mut statement = connection.prepare(
      "SELECT id, uuid, name, detail, source, repeat_type, repeat_value, tag_uuid, position, is_mandatory, is_checked, is_sent
        FROM checks
        WHERE is_sent = 0
        ORDER BY position, name",
  )?;
    let rows = statement.query_map([], row_to_check)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn count_unsent(connection: &Connection) -> Result<usize> {
    let count: i64 =
        connection.query_row("SELECT COUNT(*) FROM checks WHERE is_sent = 0", [], |row| {
            row.get(0)
        })?;
    Ok(count as usize)
}

pub fn fetch_by_uuid(connection: &Connection, uuid: &uuid::Uuid) -> Result<Option<Check>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, name, detail, source, repeat_type, repeat_value, tag_uuid, position, is_mandatory, is_checked, is_sent
         FROM checks
         WHERE uuid = ?1",
    )?;

    let check = statement
        .query_row([uuid.to_string()], row_to_check)
        .optional()?;
    Ok(check)
}

pub fn upsert(connection: &Connection, check: &Check) -> Result<i64> {
    if let Some(existing) = fetch_by_uuid(connection, &check.uuid)? {
        let source = check.source.to_storage();
        let (repeat_type, repeat_value) = check.repeat_case.to_storage();
        connection.execute(
            "UPDATE checks
             SET name = ?1, detail = ?2, source = ?3, repeat_type = ?4, repeat_value = ?5,
                 tag_uuid = ?6, position = ?7, is_mandatory = ?8, is_checked = ?9, is_sent = ?10
             WHERE uuid = ?11",
            params![
                check.name,
                check.detail,
                source,
                repeat_type,
                repeat_value,
                check.tag_uuid.map(|uuid| uuid.to_string()),
                check.position,
                bool_to_sqlite(check.is_mandatory),
                bool_to_sqlite(check.is_checked),
                bool_to_sqlite(check.is_sent),
                check.uuid.to_string(),
            ],
        )?;
        Ok(existing.id)
    } else {
        insert(connection, check)
    }
}

pub fn update(connection: &Connection, check: &Check) -> Result<()> {
    let source = check.source.to_storage();
    let (repeat_type, repeat_value) = check.repeat_case.to_storage();

    connection.execute(
        "UPDATE checks
         SET name = ?1, detail = ?2, source = ?3, repeat_type = ?4, repeat_value = ?5,
             tag_uuid = ?6, position = ?7, is_mandatory = ?8, is_checked = ?9, is_sent = ?10
         WHERE id = ?11",
        params![
            check.name,
            check.detail,
            source,
            repeat_type,
            repeat_value,
            check.tag_uuid.map(|uuid| uuid.to_string()),
            check.position,
            bool_to_sqlite(check.is_mandatory),
            bool_to_sqlite(check.is_checked),
            bool_to_sqlite(check.is_sent),
            check.id,
        ],
    )?;

    Ok(())
}

pub fn mark_sent_by_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    mark_sent_by_uuids_in_table(connection, "checks", uuids)
}

#[cfg(test)]
pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM checks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    delete_sent_missing_uuids_in_table(connection, "checks", uuids)
}

fn row_to_check(row: &Row<'_>) -> rusqlite::Result<Check> {
    let source: String = row.get(4)?;
    let repeat_type: String = row.get(5)?;
    let repeat_value = row.get(6)?;
    let tag_uuid: Option<String> = row.get(7)?;

    Ok(Check {
        id: row.get(0)?,
        uuid: parse_uuid(row.get(1)?),
        name: row.get(2)?,
        detail: row.get(3)?,
        source: CheckSourceType::from_storage(&source),
        repeat_case: CheckRepeatType::from_storage(&repeat_type, repeat_value),
        tag_uuid: tag_uuid.map(parse_uuid),
        position: row.get(8)?,
        is_mandatory: sqlite_to_bool(row.get(9)?),
        is_checked: sqlite_to_bool(row.get(10)?),
        is_sent: sqlite_to_bool(row.get(11)?),
    })
}

#[cfg(test)]
#[path = "checks_tests.rs"]
mod tests;
