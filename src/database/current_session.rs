use anyhow::{bail, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::models::CurrentSession;

const SINGLETON_ID: i64 = 1;

pub fn upsert(connection: &Connection, session: &CurrentSession) -> Result<i64> {
    connection.execute(
        "INSERT INTO current_session (id, game_uuid, game_name, turn_number, new_turn_number)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
             game_uuid = excluded.game_uuid,
             game_name = excluded.game_name,
             turn_number = excluded.turn_number,
             new_turn_number = excluded.new_turn_number",
        params![
            SINGLETON_ID,
            session.game_uuid.map(|uuid| uuid.to_string()),
            session.game_name,
            session.turn_number,
            session.new_turn_number,
        ],
    )?;

    Ok(SINGLETON_ID)
}

pub fn fetch(connection: &Connection) -> Result<Option<CurrentSession>> {
    let mut statement = connection.prepare(
        "SELECT game_uuid, game_name, turn_number, new_turn_number
         FROM current_session
         WHERE id = ?1",
    )?;

    let session = statement
        .query_row([SINGLETON_ID], row_to_current_session)
        .optional()?;
    Ok(session)
}

pub fn increment_new_turn_number_if_needed(connection: &Connection) -> Result<bool> {
    let Some(mut session) = fetch(connection)? else {
        return Ok(false);
    };

    if session.has_new_turn() {
        return Ok(false);
    }

    session.new_turn_number += 1;
    upsert(connection, &session)?;
    Ok(true)
}

pub fn validate_session_match(
    connection: &Connection,
    received_session: &Option<CurrentSession>,
) -> Result<()> {
    let Some(stored_session) = fetch(connection)? else {
        return Ok(());
    };

    let Some(stored_game_uuid) = stored_session.game_uuid else {
        return Ok(());
    };

    let Some(received_session) = received_session else {
        return Ok(());
    };

    let Some(received_game_uuid) = received_session.game_uuid else {
        return Ok(());
    };

    if stored_game_uuid != received_game_uuid {
        bail!(
            "game uuid mismatch: stored={} received={}",
            stored_game_uuid,
            received_game_uuid
        );
    }

    Ok(())
}

fn row_to_current_session(row: &Row<'_>) -> rusqlite::Result<CurrentSession> {
    let game_uuid: Option<String> = row.get(0)?;

    Ok(CurrentSession {
        game_uuid: game_uuid.map(parse_uuid),
        game_name: row.get(1)?,
        turn_number: row.get(2)?,
        new_turn_number: row.get(3)?,
    })
}

fn parse_uuid(value: String) -> Uuid {
    Uuid::parse_str(&value).unwrap_or_else(|_| Uuid::nil())
}

#[cfg(test)]
#[path = "current_session_tests.rs"]
mod tests;
