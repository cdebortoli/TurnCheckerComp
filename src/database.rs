pub mod checks;
pub mod comments;
pub mod connection;
pub mod tags;

use anyhow::Result;
use rusqlite::Connection;

#[allow(unused_imports)]
pub use connection::{
    database_path, establish_connection, establish_connection_at, establish_in_memory_connection,
};

pub fn init_database() -> Result<()> {
    let _connection = establish_connection()?;
    Ok(())
}

#[allow(dead_code)]
pub fn open_database() -> Result<Connection> {
    establish_connection()
}
