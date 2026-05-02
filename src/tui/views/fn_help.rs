use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::server::template::FUNCTIONS;
use crate::tui::app::App;

pub fn draw(f: &mut Frame, _app: &App) {
    let area = centered(f.area(), 88, 70);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Template Functions  (Press Esc or ? to close) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(inner);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Use ", Style::default().fg(Color::DarkGray)),
        Span::styled("{{function}}", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(Color::DarkGray)),
        Span::styled("{{function:arg}}", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" in Response Body. Templates are evaluated at request time.", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[0]);

    let header = Row::new(vec![
        Cell::from("Function").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        Cell::from("Syntax").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        Cell::from("Default").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        Cell::from("Description").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        Cell::from("Example output").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    ]);

    let rows: Vec<Row> = FUNCTIONS
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let style = if i % 2 == 0 {
                Style::default()
            } else {
                Style::default().bg(Color::Rgb(30, 30, 30))
            };
            Row::new(vec![
                Cell::from(Span::styled(f.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(f.syntax, Style::default().fg(Color::Yellow))),
                Cell::from(f.default_args),
                Cell::from(f.description),
                Cell::from(Span::styled(f.example_output, Style::default().fg(Color::Green))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(36),
            Constraint::Length(18),
            Constraint::Min(24),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(table, chunks[1]);
}

fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vert = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vert[1])[1]
}
