use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::server::template::FUNCTIONS;
use crate::tui::app::App;

pub fn draw(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Template Functions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(inner);

    // Header description
    let desc = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Use "),
            Span::styled("{{function}}", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("{{function:arg}}", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" in the Response Body field of a Mock."),
        ]),
        Line::from(vec![
            Span::styled(
                "Placeholders are evaluated on every incoming request.",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ])
    .alignment(Alignment::Center);
    f.render_widget(desc, chunks[0]);

    // Separator
    let sep = Paragraph::new(Line::from(
        "─".repeat(inner.width as usize),
    ))
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(sep, chunks[1]);

    // Table
    let header = Row::new(vec![
        Cell::from("Function")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED).fg(Color::Cyan)),
        Cell::from("Syntax")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED).fg(Color::Cyan)),
        Cell::from("Default")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED).fg(Color::Cyan)),
        Cell::from("Description")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED).fg(Color::Cyan)),
        Cell::from("Example output")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED).fg(Color::Cyan)),
    ])
    .height(1);

    let rows: Vec<Row> = FUNCTIONS.iter().enumerate().map(|(i, fn_doc)| {
        let bg = if i % 2 == 0 { Color::Reset } else { Color::Rgb(28, 28, 40) };
        Row::new(vec![
            Cell::from(Span::styled(
                fn_doc.name,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                fn_doc.syntax,
                Style::default().fg(Color::White),
            )),
            Cell::from(Span::styled(
                fn_doc.default_args,
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(fn_doc.description),
            Cell::from(Span::styled(
                fn_doc.example_output,
                Style::default().fg(Color::Green),
            )),
        ])
        .style(Style::default().bg(bg))
        .height(1)
    })
    .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(38),
            Constraint::Length(20),
            Constraint::Min(28),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(table, chunks[2]);

    // Footer hint
    let footer = Paragraph::new(Line::from(vec![
        hint("[1-4]", "switch tab"),
        Span::raw("  "),
        hint("[Tab]", "next tab"),
        Span::raw("  "),
        hint("[q]", "quit"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, chunks[3]);
}

fn hint(key: &str, label: &str) -> Span<'static> {
    Span::styled(
        format!("{} {}", key, label),
        Style::default().fg(Color::DarkGray),
    )
}
