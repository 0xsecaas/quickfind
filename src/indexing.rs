use crate::config::Config;
use crate::db;
use eyre::Result;
use glob::Pattern;
use rusqlite::Connection;
use std::cell::RefCell;
use std::io::{self, Write};
use std::ops::AddAssign;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use walkdir::WalkDir;

pub fn index_files(conn: &Connection, config: &Config, path: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Configuration: {:?}", config);
    }

    let start_time = Instant::now();
    let mut files_discovered = 0;
    let mut dirs_traversed = 0;
    let items_ignored = Rc::new(RefCell::new(0));
    let progress_interval = 1000; // Report progress every 1000 items
    let mut last_report_time = Instant::now();

    let ignore_patterns: Vec<Pattern> = config
        .ignore
        .iter()
        .map(|s| Pattern::new(s))
        .collect::<Result<Vec<_>, _>>()?;

    // Use the provided path for WalkDir
    let include_path_buf = PathBuf::from(path);
    let items_ignored_cloned = Rc::clone(&items_ignored);
    for entry in WalkDir::new(path)
        .max_depth(config.depth)
        .into_iter()
        .filter_entry(move |entry| {
            let entry_path = entry.path();
            let mut is_ignored = false;

            // Check against absolute path
            if ignore_patterns.iter().any(|p| p.matches_path(entry_path)) {
                is_ignored = true;
            }

            // Check against path relative to the provided path
            if !is_ignored {
                if let Ok(relative_path) = entry_path.strip_prefix(&include_path_buf) {
                    if ignore_patterns
                        .iter()
                        .any(|p| p.matches_path(relative_path))
                    {
                        is_ignored = true;
                    }
                }
            }

            if is_ignored {
                if verbose {
                    println!("Skipping ignored path: {:?}", entry_path);
                }
                items_ignored_cloned.borrow_mut().add_assign(1);
            }
            !is_ignored
        })
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(path_str) = entry_path.to_str() {
                db::insert_file(conn, path_str)?;
                files_discovered += 1;
                if verbose {
                    println!("[{}] Discovered: {}", files_discovered, path_str);
                }
            }
        } else if entry_path.is_dir() {
            dirs_traversed += 1;
        }

        let total_processed = files_discovered + dirs_traversed + *items_ignored.borrow();
        if total_processed % progress_interval == 0
            || last_report_time.elapsed().as_secs_f32() > 5.0
        {
            if verbose {
                println!(
                    "Progress: Files: {}, Dirs: {}, Ignored: {}, Elapsed: {:.2?}",
                    files_discovered,
                    dirs_traversed,
                    *items_ignored.borrow(),
                    start_time.elapsed()
                );
            } else {
                print!(
                    "\rIndexing... Files: {}, Dirs: {}, Ignored: {}, Elapsed: {:.2?}",
                    files_discovered,
                    dirs_traversed,
                    *items_ignored.borrow(),
                    start_time.elapsed()
                );
                io::stdout().flush()?;
            }
            last_report_time = Instant::now();
        }
    }

    if !verbose {
        print!("\r"); // Clear the last progress line
        io::stdout().flush()?;
    }

    if verbose {
        println!(
            "Indexing complete: Found {} files, traversed {} directories, ignored {} items in {:.2?}",
            files_discovered,
            dirs_traversed,
            *items_ignored.borrow(),
            start_time.elapsed()
        );
    } else {
        println!("Indexing complete: Found {} files, traversed {} directories, ignored {} items in {:.2?}",
            files_discovered, dirs_traversed, *items_ignored.borrow(), start_time.elapsed());
    }

    Ok(())
}
