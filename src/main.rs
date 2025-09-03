use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}};

mod cli;
mod config;
mod jira;
mod jira_api;
mod model;
mod ui;

use crate::cli::Args;
use crate::config::Config;
use crate::jira::fetch_tickets;
use crate::model::KanbanColumns;
use crate::ui::draw_ui;
use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut config = Config::load();
    
    // Handle --init flag
    if args.init {
        println!("Creating sample config at: {:?}", Config::config_path());
        let sample_config = Config::default();
        sample_config.save()?;
        println!("Config file created! Edit it and add your JIRA credentials.");
        return Ok(());
    }
    
    // Override config with CLI args
    if let Some(ref url) = args.url {
        config.jira.url = Some(url.clone());
    }
    config.query.jql = args.build_jql(&config.query.jql);
    
    // Handle --once mode (display and exit)
    if args.once {
        let tickets = fetch_tickets(&config)?;
        let columns = KanbanColumns::from_tickets(tickets);
        
        // Simple non-TUI output for use with watch
        println!("🦀 KANBARS - JIRA Board\n");
        columns.print_simple();
        return Ok(());
    }
    
    // Fetch tickets before setting up terminal
    let tickets = fetch_tickets(&config)?;
    let columns = KanbanColumns::from_tickets(tickets);
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, columns, &config, args.refresh);

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut columns: KanbanColumns,
    config: &Config,
    refresh_seconds: u64,
) -> Result<(), Box<dyn Error>> {
    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(refresh_seconds);
    let mut paused = false;
    let mut last_update_time = chrono::Local::now();
    
    loop {
        // Draw UI with current state
        terminal.draw(|f| draw_ui(f, &columns, Some(&last_update_time), paused, refresh_seconds))?;
        
        // Check for keyboard input with timeout
        let timeout = if paused {
            Duration::from_millis(100) // Short timeout when paused
        } else {
            // Calculate time until next refresh
            let elapsed = last_refresh.elapsed();
            if elapsed >= refresh_interval {
                Duration::from_millis(0) // Refresh immediately
            } else {
                refresh_interval - elapsed
            }
        };
        
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => {
                        // Manual refresh
                        match fetch_tickets(config) {
                            Ok(tickets) => {
                                columns = KanbanColumns::from_tickets(tickets);
                                last_update_time = chrono::Local::now();
                                last_refresh = Instant::now();
                            }
                            Err(e) => {
                                // TODO: Show error in UI
                                eprintln!("Refresh failed: {}", e);
                            }
                        }
                    }
                    KeyCode::Char('p') => {
                        // Toggle pause
                        paused = !paused;
                    }
                    _ => {}
                }
            }
        } else if !paused && last_refresh.elapsed() >= refresh_interval {
            // Auto-refresh
            match fetch_tickets(config) {
                Ok(tickets) => {
                    columns = KanbanColumns::from_tickets(tickets);
                    last_update_time = chrono::Local::now();
                    last_refresh = Instant::now();
                }
                Err(e) => {
                    // TODO: Show error in UI
                    eprintln!("Auto-refresh failed: {}", e);
                    last_refresh = Instant::now(); // Reset timer even on error
                }
            }
        }
    }
}