use std::collections::HashSet;

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};

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
    let mut updated = 0;
    for uuid in uuids {
        updated += connection.execute(
            "UPDATE checks SET is_sent = 1 WHERE uuid = ?1",
            [uuid.to_string()],
        )?;
    }

    Ok(updated)
}

pub fn delete(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM checks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_sent_missing_uuids(connection: &Connection, uuids: &[uuid::Uuid]) -> Result<usize> {
    let retained: HashSet<uuid::Uuid> = uuids.iter().copied().collect();
    let mut statement = connection.prepare("SELECT id, uuid FROM checks WHERE is_sent = 1")?;
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
    use crate::database::tags;
    use crate::models::check_source_type::CheckSourceType;
    use crate::models::{Check, CheckRepeatType, Tag};

    #[test]
    fn check_crud_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let tag = Tag::new("Intel", "#112233", "#FFFFFF");
        tags::insert(&connection, &tag)?;
        let mut check = Check::new("Scout");
        check.detail = Some("Reveal nearby units".to_string());
        check.source = CheckSourceType::Blueprint;
        check.repeat_case = CheckRepeatType::Conditional(3);
        check.tag_uuid = Some(tag.uuid);
        check.position = 2;
        check.is_mandatory = true;
        check.is_checked = true;
        check.is_sent = true;

        let id = super::insert(&connection, &check)?;
        check.id = id;

        let fetched = super::fetch_by_uuid(&connection, &check.uuid)?.expect("check exists");
        assert_eq!(fetched.source, CheckSourceType::Blueprint);
        assert_eq!(fetched.repeat_case, CheckRepeatType::Conditional(3));
        assert_eq!(fetched.tag_uuid, Some(tag.uuid));
        assert!(fetched.is_sent);

        check.source = CheckSourceType::Turn;
        check.repeat_case = CheckRepeatType::Until(5);
        check.name = "Scout Again".to_string();
        super::update(&connection, &check)?;

        let checks = super::fetch_all(&connection)?;
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "Scout Again");
        assert_eq!(checks[0].source, CheckSourceType::Turn);
        assert_eq!(checks[0].repeat_case, CheckRepeatType::Until(5));
        assert_eq!(checks[0].tag_uuid, Some(tag.uuid));

        super::delete(&connection, id)?;
        assert!(super::fetch_all(&connection)?.is_empty());

        Ok(())
    }

    #[test]
    fn default_source_round_trips() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let check = Check::new("Default source");

        let id = super::insert(&connection, &check)?;
        let fetched = super::fetch_by_uuid(&connection, &check.uuid)?.expect("check exists");

        assert_eq!(fetched.id, id);
        assert_eq!(fetched.source, CheckSourceType::Game);

        Ok(())
    }

    #[test]
    fn fetch_all_returns_only_global_game_and_turn_checks() -> Result<()> {
        let connection = establish_in_memory_connection()?;

        let mut global_check = Check::new("Global check");
        global_check.source = CheckSourceType::GlobalGame;

        let mut game_check = Check::new("Game check");
        game_check.source = CheckSourceType::Game;

        let mut turn_check = Check::new("Turn check");
        turn_check.source = CheckSourceType::Turn;

        super::insert(&connection, &global_check)?;
        super::insert(&connection, &game_check)?;
        super::insert(&connection, &turn_check)?;

        let mut fetched_global =
            super::fetch_by_uuid(&connection, &global_check.uuid)?.expect("global check exists");
        fetched_global.position = 99;
        super::update(&connection, &fetched_global)?;

        let mut fetched_game =
            super::fetch_by_uuid(&connection, &game_check.uuid)?.expect("game check exists");
        fetched_game.position = 2;
        super::update(&connection, &fetched_game)?;

        let mut fetched_turn =
            super::fetch_by_uuid(&connection, &turn_check.uuid)?.expect("turn check exists");
        fetched_turn.position = 1;
        super::update(&connection, &fetched_turn)?;

        let checks = super::fetch_all(&connection)?;

        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].source, CheckSourceType::GlobalGame);
        assert_eq!(checks[0].name, "Global check");
        assert_eq!(checks[1].source, CheckSourceType::Turn);
        assert_eq!(checks[1].name, "Turn check");

        Ok(())
    }

    #[test]
    fn upsert_preserves_global_game_priority_in_fetch_all() -> Result<()> {
        let connection = establish_in_memory_connection()?;

        let mut global_check = Check::new("Global check");
        global_check.source = CheckSourceType::GlobalGame;
        super::insert(&connection, &global_check)?;

        let mut game_check = Check::new("Game check");
        game_check.source = CheckSourceType::Game;
        super::insert(&connection, &game_check)?;

        let mut fetched_global =
            super::fetch_by_uuid(&connection, &global_check.uuid)?.expect("global check exists");
        fetched_global.position = 99;
        fetched_global.is_sent = true;

        let mut fetched_game =
            super::fetch_by_uuid(&connection, &game_check.uuid)?.expect("game check exists");
        fetched_game.position = 1;
        super::update(&connection, &fetched_game)?;

        super::upsert(&connection, &fetched_global)?;

        let checks = super::fetch_all(&connection)?;

        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].source, CheckSourceType::GlobalGame);
        assert_eq!(checks[0].uuid, global_check.uuid);
        assert!(checks[0].is_sent);

        Ok(())
    }

    #[test]
    fn fetch_by_source_returns_only_requested_source() -> Result<()> {
        let connection = establish_in_memory_connection()?;

        let mut game_check = Check::new("Game check");
        game_check.source = CheckSourceType::Game;

        let mut global_game_check = Check::new("Global game check");
        global_game_check.source = CheckSourceType::GlobalGame;

        let mut template_check = Check::new("Template check");
        template_check.source = CheckSourceType::Blueprint;

        super::insert(&connection, &game_check)?;
        super::insert(&connection, &global_game_check)?;
        super::insert(&connection, &template_check)?;

        let checks = super::fetch_by_source(&connection, CheckSourceType::Game)?;

        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].source, CheckSourceType::Game);
        assert_eq!(checks[0].name, "Game check");

        Ok(())
    }

    #[test]
    fn count_unsent_returns_only_unsent_checks() -> Result<()> {
        let connection = establish_in_memory_connection()?;

        let unsent = Check::new("Unsent");
        super::insert(&connection, &unsent)?;

        let mut sent = Check::new("Sent");
        sent.is_sent = true;
        super::insert(&connection, &sent)?;

        assert_eq!(super::count_unsent(&connection)?, 1);

        Ok(())
    }
}
