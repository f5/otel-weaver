// SPDX-License-Identifier: Apache-2.0

//! Command to generate a client SDK.

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::{event::{self, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders};
use tantivy::{Index, IndexWriter};
use tantivy::schema::{Schema, STORED, TEXT};
use tui_textarea::TextArea;

use weaver_logger::Logger;
use weaver_resolver::SchemaResolver;
use weaver_schema::TelemetrySchema;
use weaver_template::Error::InvalidTelemetrySchema;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;
pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

/// Parameters for the `search` command
#[derive(Parser)]
pub struct SearchParams {
    /// Schema file to resolve
    #[arg(short, long, value_name = "FILE")]
    schema: PathBuf,
}

pub struct SearchApp<'a> {
    search_area: TextArea<'a>,
    should_quit: bool,
}

/// Search for attributes and metrics in a schema file
pub fn command_search(log: &Logger, params: &SearchParams) {
    let telemetry_schema =
        SchemaResolver::resolve_schema_file(params.schema.clone(), log).map_err(|e| {
            InvalidTelemetrySchema {
                schema: params.schema.clone(),
                error: format!("{}", e),
            }
        }).unwrap_or_else(|e| {
            log.error(&format!("{}", e));
            std::process::exit(1);
        });

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("brief", TEXT | STORED);
    schema_builder.add_text_field("note", TEXT);
    let index_schema = schema_builder.build();
    let index = Index::create_in_ram(index_schema.clone());
    let mut index_writer: IndexWriter = index.writer(50_000_000).expect("Failed to create index writer");

    search_tui(&telemetry_schema).unwrap_or_else(|e| {
        log.error(&format!("{}", e));
        std::process::exit(1);
    });
}

fn search_tui(schema: &TelemetrySchema) -> Result<()> {
    // Startup
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // startup()?;
    let status = run();
    // shutdown()?;

    // Shutdown
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    status?;
    Ok(())
}

// fn startup() -> Result<()> {
//     enable_raw_mode()?;
//     execute!(std::io::stderr(), EnterAlternateScreen)?;
//     Ok(())
// }

// fn shutdown() -> Result<()> {
//     execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
//     disable_raw_mode()?;
//     Ok(())
// }

fn ui(app: &SearchApp, frame: &mut Frame<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let result_block = Block::default()
        .borders(Borders::ALL)
        .title("Search results (i.e. attributes and metrics)")
        .style(Style::default());

    let content = Paragraph::new(Text::styled(
        "Stats",
        Style::default().fg(Color::Green),
    ))
        .block(result_block);

    frame.render_widget(content, chunks[0]);

    frame.render_widget(app.search_area.widget(), chunks[1]);
}

fn update(app: &mut SearchApp) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                app.should_quit = true;
                return Ok(());
            }
        }
        app.search_area.input(event);
    }

    Ok(())
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut search_area = TextArea::default();
    search_area.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search (press `Esc` or `Ctrl-C` to stop running)"),
    );

    // application state
    let mut app = SearchApp {
        search_area,
        should_quit: false,
    };

    loop {
        // application render
        t.draw(|f| {
            ui(&app, f);
        })?;

        // application update
        update(&mut app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}