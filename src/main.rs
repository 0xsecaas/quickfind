mod config;
mod db;
mod indexing;
mod tui;

use clap::Parser;
use eyre::Result;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The search term
    search_term: Option<String>,

    /// Index files based on the configuration
    #[clap(long, short, action)]
    index: bool,

    /// Enable verbose output
    #[clap(long, short, action)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load_config()?;
    let conn = db::get_connection()?;
    db::create_tables(&conn)?;

    if cli.index {
        println!("Indexing files...");
        // Corrected call to index_files:
        // Pass conn, config, a default path (e.g., '.'), and verbose flag.
        // The actual path for indexing is handled within indexing::index_files based on config.include.
        // For the initial index command, we can pass a placeholder path like "." or the first include path.
        // However, the current indexing logic iterates through config.include internally.
        // So, we need to pass a path that makes sense for the WalkDir::new() call inside index_files.
        // The most sensible approach is to pass the first path from config.include if available, or a default.
        // For simplicity, let's assume the indexing logic will use config.include as before.
        // The 'path' argument in index_files is now used by WalkDir::new().
        // If config.include is empty, WalkDir::new(".") will be used.
        // Let's pass the first include path if it exists, otherwise a default.
        let index_path = config.include.first().map(|s| s.as_str()).unwrap_or(".");
        indexing::index_files(&conn, &config, index_path, cli.verbose)?;
        println!("Indexing complete.");
    } else {
        tui::run_tui(&conn, cli.search_term)?;
    }

    Ok(())
}
