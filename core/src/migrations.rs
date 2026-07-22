use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("../migrations/0001_init.sql"))])
}

/// Runs all pending migrations against `conn`. Safe to call on every startup.
pub fn migrate(conn: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    migrations().to_latest(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        migrations().validate().expect("migrations should be valid");
    }

    #[test]
    fn migrate_creates_expected_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        for expected in [
            "account",
            "balance_snapshot",
            "budget_target",
            "category",
            "contract",
            "merchant_rule",
            "nb_token_count",
            "position_snapshot",
            "price",
            "split",
            "sync_state",
            "transaction",
            "transfer",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "missing table: {expected}"
            );
        }
    }

    #[test]
    fn migrate_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
    }
}
