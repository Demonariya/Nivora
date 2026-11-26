//! SQLite-backed browser history persistence. Credentials are never stored here.

use rusqlite::{Connection, params};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visited_at: i64,
}

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS history (
                 id INTEGER PRIMARY KEY,
                 url TEXT NOT NULL,
                 title TEXT,
                 visited_at INTEGER NOT NULL DEFAULT (unixepoch())
             );
             CREATE INDEX IF NOT EXISTS idx_history_visited_at ON history(visited_at DESC);",
        )?;
        Ok(Self { connection })
    }

    /// Records a completed top-level navigation. Callers should exclude private sessions.
    pub fn record_visit(&self, url: &str, title: Option<&str>) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO history(url, title) VALUES (?1, ?2)",
            params![url, title],
        )?;
        Ok(())
    }

    pub fn recent_history(&self, limit: usize) -> rusqlite::Result<Vec<HistoryEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT id, url, title, visited_at
             FROM history ORDER BY visited_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([limit.min(i64::MAX as usize) as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn search_history(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut statement = self.connection.prepare(
            "SELECT id, url, title, visited_at FROM history
             WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
             ORDER BY visited_at DESC, id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![pattern, limit.min(i64::MAX as usize) as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_history_entry(&self, id: i64) -> rusqlite::Result<bool> {
        Ok(self.connection.execute("DELETE FROM history WHERE id = ?1", [id])? > 0)
    }

    pub fn clear_history(&self) -> rusqlite::Result<()> {
        self.connection.execute("DELETE FROM history", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;

    #[test]
    fn persists_and_reads_history_rows() {
        let storage = Storage::in_memory().expect("in-memory database");
        storage.record_visit("https://example.com", Some("Example")).unwrap();
        let entries = storage.recent_history(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].url, "https://example.com");
        assert_eq!(entries[0].title.as_deref(), Some("Example"));
    }

    #[test]
    fn searches_history_by_title_or_url() {
        let storage = Storage::in_memory().unwrap();
        storage.record_visit("https://example.com/docs", Some("Rust guide")).unwrap();
        storage.record_visit("https://other.test", Some("Other page")).unwrap();

        assert_eq!(storage.search_history("Rust", 10).unwrap().len(), 1);
        assert_eq!(storage.search_history("example.com", 10).unwrap().len(), 1);
        assert!(storage.search_history("missing", 10).unwrap().is_empty());
    }

    #[test]
    fn deletes_one_entry_and_can_clear_history() {
        let storage = Storage::in_memory().unwrap();
        storage.record_visit("https://one.test", None).unwrap();
        storage.record_visit("https://two.test", None).unwrap();

        let entries = storage.recent_history(10).unwrap();
        assert!(storage.delete_history_entry(entries[0].id).unwrap());
        assert!(!storage.delete_history_entry(entries[0].id).unwrap());
        assert_eq!(storage.recent_history(10).unwrap().len(), 1);

        storage.clear_history().unwrap();
        assert!(storage.recent_history(10).unwrap().is_empty());
    }
}
