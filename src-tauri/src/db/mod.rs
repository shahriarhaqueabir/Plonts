pub mod catalog;
pub mod gardens;
pub mod migrations;
pub mod seed;
pub mod settings;
use std::path::Path;

use rusqlite::Connection;

use crate::error::AppResult;

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}

/// Applies any pending migrations. Returns the number of migrations run.
pub fn migrate(conn: &mut Connection) -> AppResult<()> {
    migrations::migrations().to_latest(conn)?;
    Ok(())
}
