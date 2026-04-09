pub mod checks;
pub mod comments;
pub mod connection;
pub mod current_session;
pub mod tags;

use anyhow::Result;
use rusqlite::Connection;

#[allow(unused_imports)]
pub use connection::{
    database_path, establish_connection, establish_connection_at, establish_in_memory_connection,
    inspect_startup_state, inspect_startup_state_at, reset_database, reset_database_at,
    DatabaseStartupState,
};

#[allow(dead_code)]
pub fn open_database() -> Result<Connection> {
    establish_connection()
}
