use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::tui::app::{App, LogTab};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // tabs
            Constraint::Min(5),     // log table
            Constraint::Length(3),  // help
        ])
        .split(area);

    // ---- tab bar ----
    let tab_titles = ["Requests", "System"];
    let active_idx = match app.log_tab { LogTab::Request => 0, LogTab::System => 1 };
    let tabs = Tabs::new(tab_titles.iter().map(|t| Line::from(*t)).collect::<Vec<_>>())
        .select(active_idx)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // ---- log table ----
    match app.log_tab {
        LogTab::Request => draw_request_logs(f, app, chunks[1]),
        LogTab::System  => draw_system_logs(f, app, chunks[1]),
    }

    // ---- help ----
    let follow = if app.log_follow { "f: unfollow" } else { "f: follow" };
    let help = Paragraph::new(Line::from(vec![
        Span::raw(" ↑/↓: scroll  "),
        Span::styled("r", Style::default().fg(Color::Yellow)), Span::raw(": requests  "),
        Span::styled("s", Style::default().fg(Color::Yellow)), Span::raw(": system  "),
        Span::styled(follow, Style::default().fg(Color::Cyan)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_request_logs(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app.request_logs.iter().enumerate().map(|(i, r)| {
        let status_style = if r.response_status < 400 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };
        let row_style = if i == app.log_selected && !app.log_follow {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(r.created_at.format("%H:%M:%S%.3f").to_string()),
            Cell::from(r.port.to_string()),
            Cell::from(r.method.clone()),
            Cell::from(r.path.clone()),
            Cell::from(Span::styled(r.response_status.to_string(), status_style)),
            Cell::from(format!("{}ms", r.duration_ms)),
        ]).style(row_style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(6),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Time", "Port", "Method", "Path", "Status", "Duration"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(Block::default().title(" Request Logs ").borders(Borders::ALL));
    f.render_widget(table, area);
}

fn draw_system_logs(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app.system_logs.iter().enumerate().map(|(i, s)| {
        let level_style = match s.level.as_str() {
            "ERROR" => Style::default().fg(Color::Red),
            "WARN"  => Style::default().fg(Color::Yellow),
            "INFO"  => Style::default().fg(Color::Cyan),
            _       => Style::default().fg(Color::DarkGray),
        };
        let row_style = if i == app.log_selected && !app.log_follow {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(s.created_at.format("%H:%M:%S%.3f").to_string()),
            Cell::from(Span::styled(s.level.clone(), level_style)),
            Cell::from(s.target.clone()),
            Cell::from(s.message.clone()),
        ]).style(row_style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(6),
            Constraint::Length(24),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Time", "Level", "Target", "Message"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(Block::default().title(" System Logs ").borders(Borders::ALL));
    f.render_widget(table, area);
}
