use anyhow::Result;
use uuid::Uuid;

use crate::database::connection::establish_in_memory_connection;
use crate::models::CurrentSession;

#[test]
fn current_session_round_trip() -> Result<()> {
    let connection = establish_in_memory_connection()?;
    let game_uuid = Uuid::new_v4();
    let mut session = CurrentSession::new(Some(game_uuid), "Civ VI", 12);

    assert_eq!(super::upsert(&connection, &session)?, 1);

    let fetched = super::fetch(&connection)?.expect("current session exists");
    assert_eq!(fetched.game_uuid, Some(game_uuid));
    assert_eq!(fetched.game_name, "Civ VI");
    assert_eq!(fetched.turn_number, 12);
    assert_eq!(fetched.new_turn_number, 12);

    session.game_name = "Civ VII".to_string();
    session.turn_number = 3;
    session.new_turn_number = 4;
    super::upsert(&connection, &session)?;

    let fetched = super::fetch(&connection)?.expect("current session exists");
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
