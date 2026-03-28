use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::{Check, CheckRepeatType};

pub fn insert(connection: &Connection, check: &Check) -> Result<i64> {
    let (repeat_type, repeat_value) = check.repeat_case.to_storage();

    connection.execute(
        "INSERT INTO checks (
            uuid, name, detail, repeat_type, repeat_value, position, is_mandatory, is_checked, is_sent
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            check.uuid.to_string(),
            check.name,
            check.detail,
            repeat_type,
            repeat_value,
            check.position,
            bool_to_sqlite(check.is_mandatory),
            bool_to_sqlite(check.is_checked),
            bool_to_sqlite(check.is_sent),
        ],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn fetch_all(connection: &Connection) -> Result<Vec<Check>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, name, detail, repeat_type, repeat_value, position, is_mandatory, is_checked, is_sent
         FROM checks
         ORDER BY position, name",
    )?;
    let rows = statement.query_map([], row_to_check)?;

    let checks = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(checks)
}

pub fn fetch_by_uuid(connection: &Connection, uuid: &uuid::Uuid) -> Result<Option<Check>> {
    let mut statement = connection.prepare(
        "SELECT id, uuid, name, detail, repeat_type, repeat_value, position, is_mandatory, is_checked, is_sent
         FROM checks
         WHERE uuid = ?1",
    )?;

    let check = statement
        .query_row([uuid.to_string()], row_to_check)
        .optional()?;
    Ok(check)
}

pub fn update(connection: &Connection, check: &Check) -> Result<()> {
    let (repeat_type, repeat_value) = check.repeat_case.to_storage();

    connection.execute(
        "UPDATE checks
         SET name = ?1, detail = ?2, repeat_type = ?3, repeat_value = ?4, position = ?5,
             is_mandatory = ?6, is_checked = ?7, is_sent = ?8
         WHERE id = ?9",
        params![
            check.name,
            check.detail,
            repeat_type,
            repeat_value,
            check.position,
            bool_to_sqlite(check.is_mandatory),
            bool_to_sqlite(check.is_checked),
            bool_to_sqlite(check.is_sent),
            check.id,
        ],
    )?;

    Ok(())
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM checks WHERE id = ?1", [id])?;
    Ok(())
}

fn row_to_check(row: &Row<'_>) -> rusqlite::Result<Check> {
    let repeat_type: String = row.get(4)?;
    let repeat_value = row.get(5)?;

    Ok(Check {
        id: row.get(0)?,
        uuid: parse_uuid(row.get(1)?),
        name: row.get(2)?,
        detail: row.get(3)?,
        repeat_case: CheckRepeatType::from_storage(&repeat_type, repeat_value),
        position: row.get(6)?,
        is_mandatory: sqlite_to_bool(row.get(7)?),
        is_checked: sqlite_to_bool(row.get(8)?),
        is_sent: sqlite_to_bool(row.get(9)?),
    })
}

fn parse_uuid(value: String) -> uuid::Uuid {
    uuid::Uuid::parse_str(&value).unwrap_or_else(|_| uuid::Uuid::nil())
}

fn bool_to_sqlite(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn sqlite_to_bool(value: i64) -> bool {
    value != 0
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::database::connection::establish_in_memory_connection;
    use crate::models::{Check, CheckRepeatType};

    #[test]
    fn check_crud_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let mut check = Check::new("Scout");
        check.detail = Some("Reveal nearby units".to_string());
        check.repeat_case = CheckRepeatType::Conditional(3);
        check.position = 2;
        check.is_mandatory = true;
        check.is_checked = true;
        check.is_sent = true;

        let id = super::insert(&connection, &check)?;
        check.id = id;

        let fetched = super::fetch_by_uuid(&connection, &check.uuid)?.expect("check exists");
        assert_eq!(fetched.repeat_case, CheckRepeatType::Conditional(3));
        assert!(fetched.is_sent);

        check.repeat_case = CheckRepeatType::Until(5);
        check.name = "Scout Again".to_string();
        super::update(&connection, &check)?;

        let checks = super::fetch_all(&connection)?;
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "Scout Again");
        assert_eq!(checks[0].repeat_case, CheckRepeatType::Until(5));

        super::delete(&connection, id)?;
        assert!(super::fetch_all(&connection)?.is_empty());

        Ok(())
    }
}
