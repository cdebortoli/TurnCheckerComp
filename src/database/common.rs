use std::collections::HashSet;

use anyhow::Result;
use rusqlite::Connection;

pub(super) fn parse_uuid(value: String) -> uuid::Uuid {
    uuid::Uuid::parse_str(&value).unwrap_or_else(|_| uuid::Uuid::nil())
}

pub(super) fn bool_to_sqlite(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

pub(super) fn sqlite_to_bool(value: i64) -> bool {
    value != 0
}

pub(super) fn mark_sent_by_uuids(
    connection: &Connection,
    table: &str,
    uuids: &[uuid::Uuid],
) -> Result<usize> {
    let mut updated = 0;
    let query = format!("UPDATE {table} SET is_sent = 1 WHERE uuid = ?1");

    for uuid in uuids {
        updated += connection.execute(&query, [uuid.to_string()])?;
    }

    Ok(updated)
}

pub(super) fn delete_sent_missing_uuids(
    connection: &Connection,
    table: &str,
    uuids: &[uuid::Uuid],
) -> Result<usize> {
    let retained: HashSet<uuid::Uuid> = uuids.iter().copied().collect();
    let select_query = format!("SELECT id, uuid FROM {table} WHERE is_sent = 1");
    let delete_query = format!("DELETE FROM {table} WHERE id = ?1");
    let mut statement = connection.prepare(&select_query)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, parse_uuid(row.get::<_, String>(1)?)))
    })?;

    let mut deleted = 0;
    for row in rows {
        let (id, uuid) = row?;
        if !retained.contains(&uuid) {
            connection.execute(&delete_query, [id])?;
            deleted += 1;
        }
    }

    Ok(deleted)
}
