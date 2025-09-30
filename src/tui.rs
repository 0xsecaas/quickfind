use crate::config::load_config;
use crate::db;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use rusqlite::Connection;
use std::io::{self};
use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};
use tui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    text::{Span, Text, Spans},
};

enum Focus {
    Search,
    Results,
    HistoryModal,
    ConfirmClearHistory,
}

pub fn run_tui(conn: &Connection, initial_search: Option<String>) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut terminal, conn, initial_search, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn parse_color(s: &str) -> Option<Color> {
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" => Some(Color::Gray),
        "darkgray" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        _ => None,
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    conn: &Connection,
    initial_search: Option<String>,
    tick_rate: Duration,
) -> io::Result<()> {
    let config = load_config().unwrap_or_default();
    let highlight_color = config
        .highlight_color
        .as_ref()
        .and_then(|s| parse_color(s))
        .unwrap_or(Color::DarkGray);

    let mut last_tick = Instant::now();
    let mut search_input = initial_search.clone().unwrap_or_default();
    let mut cursor_position = 0;
    let mut error_message: Option<String> = None;

    let mut search_results = if let Some(term) = initial_search {
        db::search_files(conn, &term).unwrap_or_default()
    } else {
        vec![]
    };

    let mut results_state = ListState::default();
    results_state.select(Some(0));
    let mut focus = Focus::Search;
    let mut search_history = db::get_history(conn).unwrap_or_default();
    let mut history_state = ListState::default();

    loop {
        terminal.draw(|f| {
            ui(
                f,
                &search_input,
                &mut cursor_position,
                &search_results,
                &mut results_state,
                &focus,
                &highlight_color,
                &search_history,
                &mut history_state,
                &error_message, // Pass the error_message
            )
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match focus {
                    Focus::Search => match key.code {
                        KeyCode::Enter => {
                            if !search_input.is_empty() {
                                search_results =
                                    db::search_files(conn, &search_input).unwrap_or_default();
                                results_state.select(Some(0));
                                focus = Focus::Results;
                                if let Some(path) = search_results.get(0) {
                                    // Attempt to open the file
                                    match opener::open(path) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            // Handle file not found or other errors
                                            error_message =
                                                Some(format!("Error opening file: {}", path));
                                            eprintln!(
                                                "Failed to open file: {}. Error: {:?}",
                                                path, e
                                            );
                                            // If the error is indeed a file not found, we'd ideally want to re-index.
                                            // For now, we'll just log the error.
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Down => {
                            if !search_results.is_empty() {
                                focus = Focus::Results;
                                results_state.select(Some(0));
                            }
                        }
                        KeyCode::Backspace => {
                            if cursor_position > 0 {
                                search_input.remove(cursor_position - 1);
                                cursor_position -= 1;
                            } else if !search_input.is_empty() {
                                search_input.pop();
                            }
                            search_results =
                                db::search_files(conn, &search_input).unwrap_or_default();
                            results_state.select(Some(0));
                            error_message = None; // Clear error message on input change
                        }
                        KeyCode::Left => {
                            if cursor_position > 0 {
                                cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if cursor_position < search_input.len() {
                                cursor_position += 1;
                            }
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            focus = Focus::HistoryModal;
                            history_state.select(Some(0));
                        }
                        KeyCode::Tab => {
                            focus = Focus::Results;
                        }
                        KeyCode::Char(c) => {
                            search_input.insert(cursor_position, c);
                            cursor_position += 1;
                            search_results =
                                db::search_files(conn, &search_input).unwrap_or_default();
                            results_state.select(Some(0));
                            error_message = None; // Clear error message on input change
                        }
                        _ => {}
                    },
                    Focus::Results => match key.code {
                        KeyCode::Enter => {
                            if let Some(selected) = results_state.selected() {
                                if let Some(path) = search_results.get(selected) {
                                    db::add_to_history(conn, &search_input).unwrap_or_default();
                                    // Attempt to open the file
                                    match opener::open(path) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            // Handle file not found or other errors
                                            error_message =
                                                Some(format!("Error opening file: {}", path));
                                            eprintln!(
                                                "Failed to open file: {}. Error: {:?}",
                                                path, e
                                            );
                                            // If the error is indeed a file not found, we'd ideally want to re-index.
                                            // For now, we'll just log the error.
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('o') => {
                            if let Some(selected) = results_state.selected() {
                                if let Some(path) = search_results.get(selected) {
                                    db::add_to_history(conn, &search_input).unwrap_or_default();
                                    // Attempt to open the file
                                    match opener::open(path) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            // Handle file not found or other errors
                                            error_message =
                                                Some(format!("Error opening file: {}", path));
                                            eprintln!(
                                                "Failed to open file: {}. Error: {:?}",
                                                path, e
                                            );
                                            // If the error is indeed a file not found, we'd ideally want to re-index.
                                            // For now, we'll just log the error.
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('v') => {
                            if let Some(selected) = results_state.selected() {
                                if let Some(path) = search_results.get(selected) {
                                    db::add_to_history(conn, &search_input).unwrap_or_default();
                                    disable_raw_mode()?;
                                    execute!(io::stdout(), LeaveAlternateScreen)?;
                                    let _ = Command::new("nvim").arg(path).status();
                                    enable_raw_mode()?;
                                    execute!(io::stdout(), EnterAlternateScreen)?;
                                    terminal.clear()?;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if !search_results.is_empty() {
                                let i = match results_state.selected() {
                                    Some(i) => (i + 1) % search_results.len(),
                                    None => 0,
                                };
                                results_state.select(Some(i));
                            }
                        }
                        KeyCode::Up => {
                            if !search_results.is_empty() {
                                let i = match results_state.selected() {
                                    Some(0) => {
                                        focus = Focus::Search;
                                        0
                                    }
                                    Some(i) => {
                                        (i + search_results.len() - 1) % search_results.len()
                                    }
                                    None => 0,
                                };
                                results_state.select(Some(i));
                            }
                        }
                        KeyCode::Tab => {
                            focus = Focus::Search;
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('d') => {
                            if let Some(selected) = results_state.selected() {
                                if let Some(path) = search_results.get(selected) {
                                    let file_path = PathBuf::from(path);
                                    if let Some(dir_path) = file_path.parent() {
                                        if let Some(dir_str) = dir_path.to_str() {
                                            opener::open(dir_str).unwrap_or_else(|e| {
                                                eprintln!("Failed to open directory: {}", e);
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Focus::HistoryModal => match key.code {
                        KeyCode::Up => {
                            if !search_history.is_empty() {
                                let i = match history_state.selected() {
                                    Some(i) => {
                                        (i + search_history.len() - 1) % search_history.len()
                                    }
                                    None => 0,
                                };
                                history_state.select(Some(i));
                            }
                        }
                        KeyCode::Down => {
                            if !search_history.is_empty() {
                                let i = match history_state.selected() {
                                    Some(i) => (i + 1) % search_history.len(),
                                    None => 0,
                                };
                                history_state.select(Some(i));
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(selected) = history_state.selected() {
                                if let Some(term) = search_history.get(selected) {
                                    search_input = term.clone();
                                    search_results =
                                        db::search_files(conn, &search_input).unwrap_or_default();
                                    results_state.select(Some(0));
                                }
                            }
                            focus = Focus::Search;
                        }
                        KeyCode::Esc => {
                            focus = Focus::Search;
                        }
                        KeyCode::Delete if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            db::clear_history(conn).unwrap_or_default();
                            search_history = vec![];
                            history_state.select(None);
                            focus = Focus::Search;
                        }
                        KeyCode::Delete => {
                            focus = Focus::ConfirmClearHistory;
                        }
                        _ => {}
                    },
                    Focus::ConfirmClearHistory => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            db::clear_history(conn).unwrap_or_default();
                            search_history = vec![];
                            history_state.select(None);
                            focus = Focus::Search;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            focus = Focus::HistoryModal;
                        }
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

// Helper function to create styled spans for highlighting search terms
fn create_highlighted_spans(text: &str, term: &str, highlight_color: &Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut last_end: usize = 0;

    if term.is_empty() {
        spans.push(Span::raw(text.to_string()));
        return spans;
    }

    // Case-insensitive search for highlighting
    let term_lower = term.to_lowercase();
    let text_lower = text.to_lowercase();

    // Find all occurrences of the term (case-insensitive)
    // match_indices returns tuples of (start_index, end_index) as usize
    let indices_iter = text_lower.match_indices(&term_lower);
    let indices_iter_typed = indices_iter.map(|(start, matched_str)| (start, start + matched_str.len()));

    for (start_idx, end_idx) in indices_iter_typed {
        // Add text before the match
        if start_idx > last_end {
            spans.push(Span::raw(text[last_end..start_idx].to_string()));
        }
        // Add the matched text with highlight style
        spans.push(Span::styled(
            text[start_idx..end_idx].to_string(),
            Style::default().bg(*highlight_color),
        ));
        last_end = end_idx;
    }

    // Add any remaining text after the last match
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    spans
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    search_input: &str,
    cursor_position: &mut usize,
    search_results: &[String],
    results_state: &mut ListState,
    focus: &Focus,
    highlight_color: &Color,
    search_history: &[String],
    history_state: &mut ListState,
    error_message: &Option<String>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1), // New chunk for error message
            ]
            .as_ref(),
        )
        .split(f.size());

    let search_style = match focus {
        Focus::Search => Style::default().fg(Color::Green),
        _ => Style::default(),
    };
    let input = Paragraph::new(search_input).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(search_style),
    );
    f.render_widget(input, chunks[0]);

    if let Focus::Search = focus {
        f.set_cursor(chunks[0].x + *cursor_position as u16 + 1, chunks[0].y + 1)
    }

    let results_style = match focus {
        Focus::Results => Style::default().fg(Color::Green),
        _ => Style::default(),
    };
    let results: Vec<ListItem> = search_results
        .iter()
        .map(|item| {
            // Use the search_input for highlighting, not the whole item
            let spans = create_highlighted_spans(item, search_input, highlight_color);
            ListItem::new(Text::from(Spans::from(spans)))
        })
        .collect();

    let results_list = List::new(results)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results")
                .border_style(results_style),
        )
        .highlight_style(Style::default().bg(*highlight_color));
    f.render_stateful_widget(results_list, chunks[1], results_state);

    if let Focus::HistoryModal = focus {
        let size = f.size();
        let block = Block::default()
            .title("Search History")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(size)[1];

        let history_items: Vec<ListItem> = search_history
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();
        let history_list = List::new(history_items)
            .block(block)
            .highlight_style(Style::default().bg(Color::LightBlue));
        f.render_stateful_widget(history_list, area, history_state);
    }

    if let Focus::ConfirmClearHistory = focus {
        let size = f.size();
        let block = Block::default()
            .title("Confirm Clear History")
            .borders(Borders::ALL);
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(size)[1];

        let confirm_text =
            Paragraph::new("Are you sure you want to clear all search history? (y/N)").block(block);
        f.render_widget(confirm_text, area);
    }

    let summary_text = if search_results.is_empty() {
        "0 items".to_string()
    } else {
        format!(
            "{}/{} items",
            results_state.selected().map_or(0, |i| i + 1),
            search_results.len()
        )
    };
    let summary = Paragraph::new(summary_text).style(Style::default().fg(Color::Gray));
    f.render_widget(summary, chunks[2]);

    // Render error message if present
    if let Some(err) = error_message {
        let error_style = Style::default().fg(Color::Red);
        // Use err.as_str() to convert String to &str for Paragraph::new
        let error_paragraph = Paragraph::new(err.as_str()).style(error_style);
        f.render_widget(error_paragraph, chunks[3]);
    }
}