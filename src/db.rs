use eyre::Result;
use rusqlite::{Connection, Result as RusqliteResult, params};
use std::path::PathBuf;

pub fn get_db_path() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or_else(|| eyre::eyre!("Could not find home directory"))?;
    let db_dir = home_dir.join(".quickfind");
    std::fs::create_dir_all(&db_dir)?;
    Ok(db_dir.join("db.sqlite"))
}

pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;
    Ok(conn)
}

pub fn create_tables(conn: &Connection) -> RusqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
             id INTEGER PRIMARY KEY,
             path TEXT NOT NULL UNIQUE
         )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS search_history (
             id INTEGER PRIMARY KEY,
             term TEXT NOT NULL UNIQUE,
             timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
         )",
        [],
    )?;
    Ok(())
}

pub fn insert_file(conn: &Connection, path: &str) -> RusqliteResult<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO files (path) VALUES (?1)",
        params![path],
    )
}

pub fn add_to_history(conn: &Connection, term: &str) -> RusqliteResult<usize> {
    conn.execute(
        "INSERT OR REPLACE INTO search_history (term) VALUES (?1)",
        params![term],
    )
}

pub fn get_history(conn: &Connection) -> RusqliteResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT term FROM search_history ORDER BY timestamp DESC LIMIT 20")?;
    let mut rows = stmt.query([])?;
    let mut history = Vec::new();
    while let Some(row) = rows.next()? {
        history.push(row.get(0)?);
    }
    Ok(history)
}

pub fn clear_history(conn: &Connection) -> RusqliteResult<usize> {
    conn.execute("DELETE FROM search_history", [])
}

pub fn search_files(conn: &Connection, term: &str) -> RusqliteResult<Vec<String>> {
    let search_term = if term.starts_with('.') {
        format!("%{}", term)
    } else {
        format!("%{}%", term.replace('*', "%").replace('?', "_"))
    };
    let mut stmt = conn.prepare("SELECT path FROM files WHERE path LIKE ?1")?;
    let mut rows = stmt.query(params![search_term])?;
    let mut files = Vec::new();
    while let Some(row) = rows.next()? {
        files.push(row.get(0)?);
    }
    Ok(files)
}
