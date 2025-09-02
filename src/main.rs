use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

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
    
    // Fetch tickets before setting up terminal
    let tickets = fetch_tickets(&config)?;
    let columns = KanbanColumns::from_tickets(tickets);
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, columns);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, columns: KanbanColumns) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| draw_ui(f, &columns))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}