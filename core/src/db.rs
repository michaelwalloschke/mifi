use std::path::Path;

use rusqlite::Connection;

use crate::migrations;

/// Opens the SQLite database at `path` and runs migrations to the latest version.
/// Plain SQLite — no SQLCipher; FileVault is the encryption at rest (SPEC.md §12).
pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    migrations::migrate(&mut conn).expect("database migrations should apply cleanly");
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_and_migrates_a_fresh_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mifi.sqlite3");

        let conn = open(&path).unwrap();
        let foreign_keys: bool = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();

        assert!(foreign_keys);
        assert!(path.exists());
    }
}
