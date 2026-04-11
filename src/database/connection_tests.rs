use anyhow::Result;
use uuid::Uuid;

use super::{
    establish_connection_at, establish_in_memory_connection, insert_debug_unsent_check,
    inspect_startup_state_at, reset_database_at, DatabaseStartupState,
};
use crate::database::checks;
use crate::database::current_session;
use crate::models::{Check, CurrentSession};

#[test]
fn startup_state_detects_unsent_records() -> Result<()> {
    let temp_dir =
        std::env::temp_dir().join(format!("turn-checker-db-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let db_path = temp_dir.join("startup.db");

    assert_eq!(
        inspect_startup_state_at(db_path.clone())?,
        DatabaseStartupState::Ready
    );

    let connection = establish_connection_at(&db_path)?;
    let check = Check::new("Scout");
    checks::insert(&connection, &check)?;
    let session = CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 8);
    current_session::upsert(&connection, &session)?;

    assert_eq!(
        inspect_startup_state_at(db_path)?,
        DatabaseStartupState::NeedsUserDecision { unsent_records: 1 }
    );

    Ok(())
}

#[test]
fn startup_state_resets_existing_database_without_unsent_records() -> Result<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "turn-checker-startup-reset-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&temp_dir)?;
    let db_path = temp_dir.join("startup-reset.db");

    let connection = establish_connection_at(&db_path)?;
    let mut check = Check::new("Sent");
    check.is_sent = true;
    checks::insert(&connection, &check)?;
    let session = CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 4);
    current_session::upsert(&connection, &session)?;
    drop(connection);

    assert_eq!(
        inspect_startup_state_at(db_path.clone())?,
        DatabaseStartupState::Ready
    );

    let connection = establish_connection_at(&db_path)?;
    assert!(checks::fetch_all(&connection)?.is_empty());
    assert!(current_session::fetch(&connection)?.is_none());

    Ok(())
}

#[test]
fn reset_database_recreates_empty_schema() -> Result<()> {
    let temp_dir =
        std::env::temp_dir().join(format!("turn-checker-reset-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let db_path = temp_dir.join("reset.db");

    let connection = establish_connection_at(&db_path)?;
    let check = Check::new("Cleanup");
    checks::insert(&connection, &check)?;
    let session = CurrentSession::new(Some(Uuid::new_v4()), "Civ VI", 11);
    current_session::upsert(&connection, &session)?;
    drop(connection);

    reset_database_at(db_path.clone())?;

    let connection = establish_connection_at(&db_path)?;
    assert!(checks::fetch_all(&connection)?.is_empty());
    assert!(current_session::fetch(&connection)?.is_none());

    Ok(())
}

#[test]
fn debug_unsent_check_is_inserted_only_when_explicitly_requested() -> Result<()> {
    let connection = establish_in_memory_connection()?;

    insert_debug_unsent_check(&connection)?;
    assert_eq!(checks::fetch_all(&connection)?.len(), 1);
    assert_eq!(checks::fetch_unsent(&connection)?.len(), 1);

    insert_debug_unsent_check(&connection)?;
    assert_eq!(checks::fetch_all(&connection)?.len(), 2);

    Ok(())
}

#[test]
fn reopening_existing_database_does_not_add_rows_when_debug_seed_is_disabled() -> Result<()> {
    let temp_dir =
        std::env::temp_dir().join(format!("turn-checker-reopen-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let db_path = temp_dir.join("reopen.db");

    let connection = establish_connection_at(&db_path)?;
    assert!(checks::fetch_all(&connection)?.is_empty());
    assert!(current_session::fetch(&connection)?.is_none());
    drop(connection);

    let reopened = establish_connection_at(&db_path)?;
    assert!(checks::fetch_all(&reopened)?.is_empty());
    assert!(current_session::fetch(&reopened)?.is_none());

    Ok(())
}
