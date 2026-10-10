//! Versioned upgrades are transactional; newer databases are never downgraded.
use anyhow::{Context, Result, ensure};
use chrono::{Local, TimeZone};
use rusqlite::{Connection, OptionalExtension};

pub(crate) const VERSION: i64 = 1;
pub(crate) const KEY_ENCODING_VERSION: i64 = 1;

pub(crate) fn require_supported(conn: &Connection) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    ensure!(
        (0..=VERSION).contains(&version),
        "Unsupported database schema version {version}; use a compatible ClickPulse version. The database was not modified."
    );
    Ok(())
}

pub(crate) fn metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='clickpulse_metadata')", [], |row| row.get(0))?;
    if !exists {
        return Ok(None);
    }
    Ok(conn
        .query_row(
            "SELECT value FROM clickpulse_metadata WHERE key=?1",
            [key],
            |row| row.get(0),
        )
        .optional()?)
}

pub(crate) fn require_key_encoding(conn: &Connection) -> Result<()> {
    if let Some(value) = metadata(conn, "key_encoding_version")? {
        let version: i64 = value.parse().context("Invalid key encoding metadata")?;
        ensure!(
            version == KEY_ENCODING_VERSION,
            "Unsupported key encoding version {version}; use a compatible ClickPulse version."
        );
    }
    Ok(())
}

pub(crate) fn legacy_cutoff(conn: &Connection) -> Result<i64> {
    if let Some(value) = metadata(conn, "legacy_key_cutoff_bucket")? {
        return value
            .parse()
            .context("Invalid legacy key encoding boundary");
    }
    // Compatibility only for the original unversioned recorder. New databases
    // never use this date. An owned upgrade stores an absolute bucket once;
    // cached historical eras and explicit metadata always take precedence.
    Ok(Local
        .with_ymd_and_hms(2026, 6, 5, 12, 46, 0)
        .earliest()
        .context("Invalid legacy key encoding date")?
        .timestamp()
        .div_euclid(60))
}
