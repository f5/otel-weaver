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
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tui_textarea::TextArea;

use weaver_logger::Logger;
use weaver_resolver::SchemaResolver;
use weaver_semconv::SemConvCatalog;

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
    sem_conv_catalog: SemConvCatalog,
    search_area: TextArea<'a>,

    results: StatefulResults,

    searcher: tantivy::Searcher,
    query_parser: tantivy::query::QueryParser,
    current_query: Option<String>,

    should_quit: bool,
}

/// A result item
pub struct ResultItem {
    r#type: String,
    id: String,
    brief: String,
}

/// A stateful list of items
pub struct StatefulResults {
    state: ListState,
    items: Vec<ResultItem>,
}

impl StatefulResults {
    /// Creates a new stateful list of items
    fn new() -> StatefulResults {
        StatefulResults {
            state: ListState::default(),
            items: vec![],
        }
    }

    /// Selects the next item in the list
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Selects the previous item in the list
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Unselects the current selection
    fn unselect(&mut self) {
        self.state.select(None);
    }

    /// Clears the results
    fn clear(&mut self) {
        self.unselect();
        self.items.clear();
    }
}

/// Search for attributes and metrics in a schema file
pub fn command_search(log: impl Logger + Sync + Clone, params: &SearchParams) {
    let schema = SchemaResolver::load_schema_from_path(params.schema.clone(), log.clone())
        .unwrap_or_else(|e| {
            log.error(&format!("{}", e));
            std::process::exit(1);
        });
    let sem_conv_catalog = SchemaResolver::semantic_catalog_from_schema(&schema, log.clone())
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
    for attr in sem_conv_catalog.attributes_iter() {
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
    for metric in sem_conv_catalog.metrics_iter() {
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
        sem_conv_catalog,
        search_area,
        results: StatefulResults::new(),
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
                    let r#type = values[0].value().as_text().unwrap_or_default();
                    let id = values[1].value().as_text().unwrap_or_default();
                    let brief = values[2].value().as_text().unwrap_or_default();

                    app.results.items.push(ResultItem {
                        r#type: r#type.to_string(),
                        id: id.to_string(),
                        brief: brief.to_string(),
                    });
                }
                app.results.next();
            }
            Err(_e) => {
                app.results.clear();
            }
        }
    });

    let items: Vec<ListItem> = app
        .results
        .items
        .iter()
        .map(|item| {
            ListItem::new(Line::from(Span::styled(
                format!("{: <10} {: <30} {}", item.r#type, item.id, item.brief),
                Style::default().fg(Color::White),
            )))
        })
        .collect();

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(outer_layout[0]);

    let result_block = Block::default()
        .borders(Borders::ALL)
        .title("Search results (i.e. attributes and metrics)")
        .style(Style::default());

    let content = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .block(result_block);

    frame.render_stateful_widget(content, inner_layout[0], &mut app.results.state);

    // Detail area
    let item = match app.results.state.selected() {
        Some(i) => app.results.items.get(i),
        None => None,
    };
    frame.render_widget(detail_area(app, item), inner_layout[1]);

    frame.render_widget(app.search_area.widget(), outer_layout[1]);
}

fn detail_area<'a>(app: &'a SearchApp<'a>, item: Option<&'a ResultItem>) -> Paragraph<'a> {
    let paragraph = if let Some(item) = item {
        let text = match item.r#type.as_str() {
            "attribute" => {
                let attribute = app.sem_conv_catalog.get_attribute(item.id.as_str()).expect("Failed to get attribute (fix me)");
                vec![
                    Line::from(vec![
                        Span::styled("Type   : ", Style::default().fg(Color::Yellow)),
                        Span::raw("Attribute"),
                    ]),
                    Line::from(vec![
                        Span::styled("Id     : ", Style::default().fg(Color::Yellow)),
                        Span::raw(attribute.id()),
                    ]),
                    Line::from(vec![
                        Span::styled("Brief  : ", Style::default().fg(Color::Yellow)),
                        Span::raw(attribute.brief()),
                    ]),
                    Line::from(vec![
                        Span::styled("Note   : ", Style::default().fg(Color::Yellow)),
                        Span::raw(attribute.note()),
                    ]),
                ]
            },
            "metric" => {
                let metric = app.sem_conv_catalog.get_metric(item.id.as_str()).expect("Failed to get metric (fix me)");
                vec![
                    Line::from(vec![
                        Span::styled("Type   : ", Style::default().fg(Color::Yellow)),
                        Span::raw("Metric"),
                    ]),
                    Line::from(vec![
                        Span::styled("Name   : ", Style::default().fg(Color::Yellow)),
                        Span::raw(metric.name.clone()),
                    ]),
                    Line::from(vec![
                        Span::styled("Brief  : ", Style::default().fg(Color::Yellow)),
                        Span::raw(metric.brief.clone()),
                    ]),
                    Line::from(vec![
                        Span::styled("Note   : ", Style::default().fg(Color::Yellow)),
                        Span::raw(metric.note.clone()),
                    ]),
                    Line::from(vec![
                        Span::styled("Unit   : ", Style::default().fg(Color::Yellow)),
                        Span::raw(metric.unit.clone().unwrap_or_default()),
                    ]),
                ]
            },
            _ => vec![]
        };
        Paragraph::new(text).style(Style::default().fg(Color::Gray))
    } else {
        Paragraph::new(vec![Line::default()])
    };
    paragraph.block(Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .style(Style::default()))
        .wrap(Wrap { trim: true })
}

fn update(app: &mut SearchApp) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    KeyCode::Up => app.results.previous(),
                    KeyCode::Down => app.results.next(),
                    _ => {}
                }
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
