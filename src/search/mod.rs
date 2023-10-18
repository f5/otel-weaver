// SPDX-License-Identifier: Apache-2.0

//! Command to generate a client SDK.

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap};
use ratatui::widgets::Cell;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, STORED, TEXT};
use tui_textarea::TextArea;

use weaver_logger::Logger;
use weaver_resolver::SchemaResolver;
use weaver_schema::TelemetrySchema;

mod semconv;
mod schema;

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
    schema: TelemetrySchema,
    search_area: TextArea<'a>,

    results: StatefulResults,

    searcher: tantivy::Searcher,
    query_parser: tantivy::query::QueryParser,
    current_query: Option<String>,

    should_quit: bool,
}

/// A result item
pub struct ResultItem {
    source: String,
    r#type: String,
    id: String,
    brief: String,
}

/// A stateful list of items
pub struct StatefulResults {
    state: TableState,
    // ListState,
    items: Vec<ResultItem>,
}

impl StatefulResults {
    /// Creates a new stateful list of items
    fn new() -> StatefulResults {
        StatefulResults {
            state: TableState::default(), // ListState::default(),
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
    let schema = SchemaResolver::resolve_schema_file(params.schema.clone(), log.clone())
        .unwrap_or_else(|e| {
            log.error(&format!("{}", e));
            std::process::exit(1);
        });
    let sem_conv_catalog = schema.semantic_convention_catalog();

    let mut schema_builder = Schema::builder();

    let source = schema_builder.add_text_field("source", TEXT | STORED);
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
                source => "semconv",
                r#type => "attribute",
                id => attr.id(),
                brief => attr.brief(),
                note => attr.note()
            ))
            .expect("Failed to add document");
    }

    // Index metric groups
    for metric in sem_conv_catalog.metrics_iter() {
        index_writer
            .add_document(doc!(
                source => "semconv",
                r#type => "metric",
                id => metric.name(),
                brief => metric.brief(),
                note => metric.note()
            ))
            .expect("Failed to add document");
    }

    // Index metrics
    for metric in schema.metrics() {
        index_writer
            .add_document(doc!(
                source => "schema",
                r#type => "metric",
                id => metric.name(),
                brief => metric.brief(),
                note => metric.note()
            ))
            .expect("Failed to add document");
    }
    for metric_group in schema.metric_groups() {
        index_writer
            .add_document(doc!(
                source => "schema",
                r#type => "metric_group",
                id => metric_group.id(),
                brief => "",
                note => ""
            ))
            .expect("Failed to add document");
    }

    // Index events
    for event in schema.events() {
        index_writer
            .add_document(doc!(
                source => "schema",
                r#type => "event",
                id => event.event_name.clone(),
                brief => event.domain.clone(),
                note => ""
            ))
            .expect("Failed to add document");
    }

    // Index spans
    for span in schema.spans() {
        index_writer
            .add_document(doc!(
                source => "schema",
                r#type => "span",
                id => span.span_name.clone(),
                brief => "",
                note => ""
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
    let query_parser = QueryParser::for_index(&index, vec![source, r#type, id, brief, note]);

    let mut search_area = TextArea::default();
    search_area.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Search (press `Esc` or `Ctrl-C` to stop running) ")
            .title_style(Style::default().fg(Color::Yellow)),
    );

    // application state
    let mut app = SearchApp {
        schema,
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
                    let source = values[0].value().as_text().unwrap_or_default();
                    let r#type = values[1].value().as_text().unwrap_or_default();
                    let id = values[2].value().as_text().unwrap_or_default();
                    let brief = values[3].value().as_text().unwrap_or_default();

                    app.results.items.push(ResultItem {
                        source: source.to_string(),
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

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default();
    let header_cells = ["source:", "type:", "name:", "brief:"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    let rows: Vec<Row> = app
        .results
        .items
        .iter()
        .map(|item| {
            let cells = vec![
                Cell::from(item.source.clone()),
                Cell::from(item.r#type.clone()),
                Cell::from(item.id.clone()),
                Cell::from(item.brief.clone()),
            ];
            Row::new(cells).height(1).bottom_margin(0)
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

    let content = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Search results ").title_style(Style::default().fg(Color::Yellow)))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Max(8),
            Constraint::Max(12),
            Constraint::Max(30),
            Constraint::Max(120),
        ]);

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
        let source = item.source.as_str();
        let r#type = item.r#type.as_str();

        match (source, r#type) {
            ("semconv", "attribute") => semconv::attribute::widget(app.schema.semantic_convention_catalog().attribute(item.id.as_str())),
            ("semconv", "metric") => semconv::metric::widget(app.schema.semantic_convention_catalog().metric(item.id.as_str())),
            ("schema", "metric") => schema::metric::widget(app.schema.metric(item.id.as_str())),
            ("schema", "metric_group") => Paragraph::new(vec![Line::default()]),
            ("schema", "event") => Paragraph::new(vec![Line::default()]),
            ("schema", "span") => schema::span::widget(app.schema.span(item.id.as_str())),
            _ => Paragraph::new(vec![Line::default()]),
        }
    } else {
        Paragraph::new(vec![Line::default()])
    };

    paragraph
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .title_style(Style::default().fg(Color::Yellow))
                .style(Style::default()),
        )
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
