use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::path::PathBuf;

pub fn get_db_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".notectl")
}

pub fn get_db_path() -> PathBuf {
    get_db_dir().join("notes.db")
}

pub fn open_connection() -> SqlResult<Connection> {
    let db_dir = get_db_dir();
    fs::create_dir_all(&db_dir).expect("Could not create ~/.notectl directory");
    let conn = Connection::open(get_db_path())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

pub fn initialize(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            category TEXT,
            is_daily BOOLEAN DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS tags (
            note_id INTEGER NOT NULL,
            tag TEXT NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_tags_note_id ON tags(note_id);
        CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            task TEXT NOT NULL,
            completed BOOLEAN DEFAULT 0,
            priority TEXT DEFAULT 'medium',
            due_date INTEGER,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS templates (
            name TEXT PRIMARY KEY,
            content TEXT NOT NULL
        );
        ",
    )?;

    // Create FTS table if it doesn't exist
    // We use a separate check because CREATE VIRTUAL TABLE IF NOT EXISTS
    // doesn't work reliably across all SQLite versions
    let fts_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='notes_fts'",
        [],
        |row| row.get(0),
    )?;

    if !fts_exists {
        conn.execute_batch(
            "CREATE VIRTUAL TABLE notes_fts USING fts5(content, content_rowid=id);",
        )?;
    }

    Ok(())
}
