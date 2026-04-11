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
        "SELECT id, game_uuid, game_name, turn_number, new_turn_number
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
    let game_uuid: Option<String> = row.get(1)?;

    Ok(CurrentSession {
        id: row.get(0)?,
        game_uuid: game_uuid.map(parse_uuid),
        game_name: row.get(2)?,
        turn_number: row.get(3)?,
        new_turn_number: row.get(4)?,
    })
}

fn parse_uuid(value: String) -> Uuid {
    Uuid::parse_str(&value).unwrap_or_else(|_| Uuid::nil())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use uuid::Uuid;

    use crate::database::connection::establish_in_memory_connection;
    use crate::models::CurrentSession;

    #[test]
    fn current_session_round_trip() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        let game_uuid = Uuid::new_v4();
        let mut session = CurrentSession::new(Some(game_uuid), "Civ VI", 12);

        let id = super::upsert(&connection, &session)?;
        session.id = id;

        let fetched = super::fetch(&connection)?.expect("current session exists");
        assert_eq!(fetched.id, 1);
        assert_eq!(fetched.game_uuid, Some(game_uuid));
        assert_eq!(fetched.game_name, "Civ VI");
        assert_eq!(fetched.turn_number, 12);
        assert_eq!(fetched.new_turn_number, 12);

        session.game_name = "Civ VII".to_string();
        session.turn_number = 3;
        session.new_turn_number = 4;
        super::upsert(&connection, &session)?;

        let fetched = super::fetch(&connection)?.expect("current session exists");
        assert_eq!(fetched.id, 1);
        assert_eq!(fetched.game_uuid, Some(game_uuid));
        assert_eq!(fetched.game_name, "Civ VII");
        assert_eq!(fetched.turn_number, 3);
        assert_eq!(fetched.new_turn_number, 4);

        Ok(())
    }

    #[test]
    fn increment_new_turn_number_if_needed_increments_once() -> Result<()> {
        let connection = establish_in_memory_connection()?;
        super::upsert(
            &connection,
            &CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 12),
        )?;

        assert!(super::increment_new_turn_number_if_needed(&connection)?);
        assert!(!super::increment_new_turn_number_if_needed(&connection)?);

        let fetched = super::fetch(&connection)?.expect("current session exists");
        assert_eq!(fetched.turn_number, 12);
        assert_eq!(fetched.new_turn_number, 13);
        assert!(fetched.has_new_turn());

        Ok(())
    }

    // #[test]
    // fn game_uuid_validation_ignores_missing_stored_uuid() -> Result<()> {
    //     let connection = establish_in_memory_connection()?;
    //     let session = CurrentSession::new(None, "Humankind", 4);
    //     super::upsert(&connection, &session)?;

    //     super::validate_game_uuid_match(&connection, Some(Uuid::new_v4()))?;

    //     Ok(())
    // }

    // #[test]
    // fn game_uuid_validation_rejects_mismatch() -> Result<()> {
    //     let connection = establish_in_memory_connection()?;
    //     let session = CurrentSession::new(Some(Uuid::new_v4()), "Humankind", 4);
    //     super::upsert(&connection, &session)?;

    //     let error = super::validate_game_uuid_match(&connection, Some(Uuid::new_v4()))
    //         .expect_err("mismatch should fail");
    //     assert!(error.to_string().contains("game uuid mismatch"));

    //     Ok(())
    // }
}
