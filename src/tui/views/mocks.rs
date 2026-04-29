use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(10), Constraint::Length(3)])
        .split(area);

    // ---- mock list ----
    let rows: Vec<Row> = app.mocks.iter().enumerate().map(|(i, m)| {
        let enabled_sym = if m.enabled { "●" } else { "○" };
        let enabled_style = if m.enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) };
        let style = if i == app.mock_selected {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(m.id.to_string()),
            Cell::from(m.method.to_string()),
            Cell::from(m.path.clone()),
            Cell::from(m.name.clone()),
            Cell::from(Span::styled(enabled_sym, enabled_style)),
        ]).style(style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Min(15),
            Constraint::Length(4),
        ],
    )
    .header(
        Row::new(vec!["ID", "Method", "Path", "Name", "On"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(Block::default().title(" Mocks ").borders(Borders::ALL));
    f.render_widget(table, chunks[0]);

    // ---- detail pane ----
    let detail = if let Some(m) = app.selected_mock() {
        let headers = serde_json::to_string(&m.response_headers).unwrap_or_default();
        format!(
            "Status: {}  Delay: {}ms\nHeaders: {}\nBody: {}",
            m.response_status,
            m.response_delay_ms,
            headers,
            &m.response_body.chars().take(200).collect::<String>(),
        )
    } else {
        "No mock selected".to_owned()
    };
    let detail_widget = Paragraph::new(detail)
        .block(Block::default().title(" Detail ").borders(Borders::ALL));
    f.render_widget(detail_widget, chunks[1]);

    // ---- help ----
    let help = Paragraph::new(Line::from(vec![
        Span::raw(" ↑/↓: navigate  "),
        Span::styled("n", Style::default().fg(Color::Yellow)), Span::raw(": new  "),
        Span::styled("e", Style::default().fg(Color::Yellow)), Span::raw(": edit  "),
        Span::styled("d", Style::default().fg(Color::Red)),    Span::raw(": delete  "),
        Span::styled("Space", Style::default().fg(Color::Cyan)), Span::raw(": toggle "),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

const MOCK_LABELS: &[&str] = &[
    "Port ID", "Method", "Path", "Name", "Description",
    "Response Status", "Delay (ms)", "Response Body",
];

pub fn draw_modal(f: &mut Frame, app: &App) {
    use ratatui::widgets::Clear;
    use crate::tui::app::ModalKind;

    let is_edit = matches!(app.modal, Some(ModalKind::MockEdit));
    let title = if is_edit { " Edit Mock " } else { " New Mock " };

    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default().title(title).borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block.clone(), area);

    let constraints: Vec<Constraint> = MOCK_LABELS
        .iter()
        .map(|_| Constraint::Length(3))
        .chain(std::iter::once(Constraint::Min(1)))
        .collect();

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(area);

    for (i, label) in MOCK_LABELS.iter().enumerate() {
        let value = app.modal_fields.get(i).map(|s| s.as_str()).unwrap_or("");
        let is_active = app.modal_field_idx == i;
        let border_style = if is_active { Style::default().fg(Color::Cyan) } else { Style::default() };
        let widget = Paragraph::new(value)
            .block(Block::default().title(*label).borders(Borders::ALL).border_style(border_style));
        f.render_widget(widget, inner[i]);
    }

    let hint = Paragraph::new("Tab: next  Enter: save  Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));
    if let Some(last) = inner.last() {
        f.render_widget(hint, *last);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
