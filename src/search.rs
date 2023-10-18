// SPDX-License-Identifier: Apache-2.0

//! Command to generate a client SDK.

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tui_textarea::TextArea;

use weaver_logger::Logger;
use weaver_resolver::SchemaResolver;

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
    results: Vec<ListItem<'a>>,

    searcher: tantivy::Searcher,
    query_parser: tantivy::query::QueryParser,
    current_query: Option<String>,

    should_quit: bool,
}

/// Search for attributes and metrics in a schema file
pub fn command_search(log: impl Logger + Sync + Clone, params: &SearchParams) {
    let schema = SchemaResolver::load_schema_from_path(params.schema.clone(), log.clone())
        .unwrap_or_else(|e| {
            log.error(&format!("{}", e));
            std::process::exit(1);
        });
    let catalog = SchemaResolver::semantic_catalog_from_schema(&schema, log.clone())
        .unwrap_or_else(|e| {
            log.error(&format!("{}", e));
            std::process::exit(1);
        });

    let mut schema_builder = Schema::builder();

    let r#type = schema_builder.add_text_field("type", TEXT | STORED);
    let id = schema_builder.add_text_field("id", TEXT | STORED);
    let brief = schema_builder.add_text_field("brief", TEXT | STORED);
    let note = schema_builder.add_text_field("note", TEXT);

    let index_schema = schema_builder.build();
    let index = Index::create_in_ram(index_schema.clone());
    let mut index_writer: IndexWriter = index
        .writer(10_000_000)
        .expect("Failed to create index writer");

    // Index attributes
    for attr in catalog.attributes_iter() {
        index_writer
            .add_document(doc!(
                r#type => "attribute",
                id => attr.id(),
                brief => attr.brief(),
                note => attr.note()
            ))
            .expect("Failed to add document");
    }

    // Index metrics
    for metric in catalog.metrics_iter() {
        index_writer
            .add_document(doc!(
                r#type => "metric",
                id => metric.name(),
                brief => metric.brief(),
                note => metric.note()
            ))
            .expect("Failed to add document");
    }

    index_writer
        .commit()
        .expect("Failed to commit index writer");
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
        .expect("Failed to create reader");
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![r#type, id, brief, note]);

    let mut search_area = TextArea::default();
    search_area.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search (press `Esc` or `Ctrl-C` to stop running)"),
    );

    // application state
    let mut app = SearchApp {
        search_area,
        results: vec![],
        searcher,
        query_parser,
        current_query: None,
        should_quit: false,
    };

    search_tui(&mut app).unwrap_or_else(|e| {
        log.error(&format!("{}", e));
        std::process::exit(1);
    });
}

fn search_tui(app: &mut SearchApp<'_>) -> Result<()> {
    // Startup
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let status = run(app);

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

fn ui(app: &mut SearchApp, frame: &mut Frame<'_>) {
    app.search_area.lines().iter().for_each(|query| {
        if let Some(current_query) = app.current_query.as_ref() {
            if current_query == query {
                return;
            }
        }
        app.current_query = Some(query.to_string());
        match app.query_parser.parse_query(query) {
            Ok(query) => {
                app.results.clear();
                let top_docs = app
                    .searcher
                    .search(&query, &TopDocs::with_limit(100))
                    .expect("Failed to search");
                for (_score, doc_address) in top_docs {
                    let retrieved_doc = app
                        .searcher
                        .doc(doc_address)
                        .expect("Failed to retrieve document");
                    let values = retrieved_doc.field_values();
                    app.results.push(ListItem::new(Line::from(Span::styled(
                        format!(
                            "{: <10} {: <25} {}",
                            values[0].value().as_text().unwrap_or_default(),
                            values[1].value().as_text().unwrap_or_default(),
                            values[2].value().as_text().unwrap_or_default()
                        ),
                        Style::default().fg(Color::Yellow),
                    ))));
                }
            }
            Err(_e) => {
                app.results.clear();
            }
        }
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let result_block = Block::default()
        .borders(Borders::ALL)
        .title("Search results (i.e. attributes and metrics)")
        .style(Style::default());

    let content = List::new(app.results.clone()).block(result_block);

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

fn run(app: &mut SearchApp<'_>) -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    loop {
        // application render
        t.draw(|f| {
            ui(app, f);
        })?;

        // application update
        update(app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
