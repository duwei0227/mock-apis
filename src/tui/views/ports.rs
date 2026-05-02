use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let rows: Vec<Row> = app.ports.iter().enumerate().map(|(i, p)| {
        let running = app.running_port_ids.contains(&p.id);
        let status_style = if running {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };
        let status = if running { "● Running" } else { "○ Stopped" };
        let style = if i == app.port_selected {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(p.id.to_string()),
            Cell::from(p.port.to_string()),
            Cell::from(p.label.clone()),
            Cell::from(Span::styled(status, status_style)),
        ])
        .style(style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(20),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["ID", "Port", "Label", "Status"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(Block::default().title(" Ports ").borders(Borders::ALL));

    f.render_widget(table, chunks[0]);

    let help = Paragraph::new(Line::from(vec![
        Span::raw(" ↑/↓: navigate  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(": new  "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(": edit  "),
        Span::styled("d", Style::default().fg(Color::Red)),
        Span::raw(": delete  "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(": toggle port on/off  "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(": quit "),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[1]);
}

/// Render the port create/edit modal.
pub fn draw_modal(f: &mut Frame, app: &App) {
    use ratatui::widgets::Clear;
    use crate::tui::app::ModalKind;

    let is_edit = matches!(app.modal, Some(ModalKind::PortEdit));
    let title = if is_edit { " Edit Port " } else { " New Port " };
    let labels = ["Port number *", "Label"];

    let area = centered_rect(40, 50, f.area());
    f.render_widget(Clear, area);

    let block = Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            std::iter::repeat(Constraint::Length(3))
                .take(labels.len())
                .chain(std::iter::once(Constraint::Min(1)))
                .collect::<Vec<_>>()
        )
        .split(area);

    for (i, label) in labels.iter().enumerate() {
        let value = app.modal_fields.get(i).map(|s| s.as_str()).unwrap_or("");
        let is_active = app.modal_field_idx == i;
        let border_style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        let content = if is_active {
            let chars: Vec<char> = value.chars().collect();
            let cur = app.modal_cursor_pos.min(chars.len());
            let before: String = chars[..cur].iter().collect();
            let cursor_ch = chars.get(cur).copied().unwrap_or(' ');
            let after: String = chars[cur.saturating_add(1).min(chars.len())..].iter().collect();
            Line::from(vec![
                Span::raw(before),
                Span::styled(cursor_ch.to_string(), Style::default().add_modifier(Modifier::REVERSED)),
                Span::raw(after),
            ])
        } else {
            Line::from(value)
        };
        let widget = Paragraph::new(content)
            .block(Block::default().title(*label).borders(Borders::ALL).border_style(border_style));
        f.render_widget(widget, inner[i]);
    }

    let (hint_text, hint_style) = if let Some(err) = &app.modal_error {
        (err.as_str(), Style::default().fg(Color::Red))
    } else if app.cancel_confirm_pending {
        ("Discard changes?  Enter: yes  Esc: no", Style::default().fg(Color::Yellow))
    } else {
        ("Tab: next field  Enter: save  Esc: cancel", Style::default().fg(Color::DarkGray))
    };
    let hint = Paragraph::new(hint_text).style(hint_style);
    if let Some(last) = inner.last() {
        f.render_widget(hint, *last);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}
