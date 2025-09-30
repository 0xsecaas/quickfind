use eyre::Result;
use rusqlite::{params, Connection, Result as RusqliteResult};
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
    Ok(())
}

pub fn insert_file(conn: &Connection, path: &str) -> RusqliteResult<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO files (path) VALUES (?1)",
        params![path],
    )
}

// Updated search_files function to handle specific search patterns.
pub fn search_files(conn: &Connection, term: &str) -> RusqliteResult<Vec<String>> {
    let mut files = Vec::new();
    let mut stmt;

    if term.starts_with('.') {
        // General case for terms starting with '.' (e.g., '.mp3', '.config')
        // Match paths that END with the term (e.g., '.config' matches 'my/path/.config').
        // We assume terms starting with '.' are literal and do not contain SQL wildcards.
        let search_term = format!("%{}", term);
        stmt = conn.prepare("SELECT path FROM files WHERE path LIKE ?1")?;
        let mut rows = stmt.query(params![search_term])?;
        while let Some(row) = rows.next()? {
            files.push(row.get(0)?);
        }
    } else {
        // General case for terms not starting with '.'
        // Split the term into words and search for each word independently.
        let search_words: Vec<String> = term
            .split_whitespace()
            .map(|s| s.replace('*', "%").replace('?', "_").to_lowercase())
            .collect();

        if search_words.is_empty() {
            return Ok(vec![]);
        }

        let mut query = "SELECT path FROM files WHERE ".to_string();
        let mut params_vec: Vec<String> = Vec::new();

        for (i, word) in search_words.iter().enumerate() {
            if i > 0 {
                query.push_str(" AND ");
            }
            query.push_str(&format!("LOWER(path) LIKE ?{}", i + 1));
            params_vec.push(format!("%{}%", word));
        }

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params_vec))?;
        while let Some(row) = rows.next()? {
            files.push(row.get(0)?);
        }
    }
    Ok(files)
}

