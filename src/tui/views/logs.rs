use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap},
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
        Span::raw("  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)), Span::raw(": detail  "),
        Span::styled("Esc", Style::default().fg(Color::DarkGray)), Span::raw(": close detail"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);

    // ---- detail overlay ----
    if app.log_detail_open && app.log_tab == LogTab::Request {
        if let Some(log) = app.request_logs.get(app.log_selected) {
            draw_request_detail(f, log, area);
        }
    }
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
        let ip = r.client_ip.as_deref().unwrap_or("-");
        Row::new(vec![
            Cell::from(r.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::from(r.port.to_string()),
            Cell::from(ip.to_owned()),
            Cell::from(r.method.clone()),
            Cell::from(r.path.clone()),
            Cell::from(Span::styled(r.response_status.to_string(), status_style)),
            Cell::from(format!("{}ms", r.duration_ms)),
        ]).style(row_style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(19),
            Constraint::Length(6),
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(6),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Time", "Port", "IP", "Method", "Path", "Status", "Duration"])
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
            Cell::from(s.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::from(Span::styled(s.level.clone(), level_style)),
            Cell::from(s.target.clone()),
            Cell::from(s.message.clone()),
        ]).style(row_style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(19),
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

fn draw_request_detail(f: &mut Frame, log: &crate::models::RequestLog, area: Rect) {
    let popup_area = centered_rect(80, 85, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Request Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Build detail text.
    let req_headers: Vec<String> = log.request_headers.iter()
        .map(|(k, v)| format!("  {}: {}", k, v))
        .collect();
    let resp_headers: Vec<String> = log.response_headers.iter()
        .map(|(k, v)| format!("  {}: {}", k, v))
        .collect();

    let mut lines: Vec<Line> = Vec::new();

    // ---- Request section ----
    lines.push(Line::from(Span::styled("── Request ──────────────────────────", Style::default().fg(Color::Yellow))));
    lines.push(Line::from(vec![
        Span::styled("  Method: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(log.method.clone()),
        Span::raw("  "),
        Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(log.path.clone()),
    ]));
    if let Some(qs) = &log.query_string {
        lines.push(Line::from(vec![
            Span::styled("  Query: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(qs.clone()),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("  Client IP: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(log.client_ip.as_deref().unwrap_or("-").to_owned()),
        Span::raw("  "),
        Span::styled("Port: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(log.port.to_string()),
        Span::raw("  "),
        Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(log.created_at.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()),
    ]));
    lines.push(Line::from(Span::styled("  Request Headers:", Style::default().add_modifier(Modifier::BOLD))));
    if req_headers.is_empty() {
        lines.push(Line::from("    (none)"));
    } else {
        for h in &req_headers {
            lines.push(Line::from(h.as_str()));
        }
    }
    lines.push(Line::from(Span::styled("  Request Body:", Style::default().add_modifier(Modifier::BOLD))));
    match &log.request_body {
        Some(b) if !b.is_empty() => lines.push(Line::from(format!("  {}", b))),
        _ => lines.push(Line::from("    (empty)")),
    }

    // ---- Response section ----
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("── Response ─────────────────────────", Style::default().fg(Color::Green))));
    let status_style = if log.response_status < 400 {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };
    lines.push(Line::from(vec![
        Span::styled("  Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(log.response_status.to_string(), status_style),
        Span::raw("  "),
        Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}ms", log.duration_ms)),
    ]));
    lines.push(Line::from(Span::styled("  Response Headers:", Style::default().add_modifier(Modifier::BOLD))));
    if resp_headers.is_empty() {
        lines.push(Line::from("    (none)"));
    } else {
        for h in &resp_headers {
            lines.push(Line::from(h.as_str()));
        }
    }
    lines.push(Line::from(Span::styled("  Response Body:", Style::default().add_modifier(Modifier::BOLD))));
    match &log.response_body {
        Some(b) if !b.is_empty() => lines.push(Line::from(format!("  {}", b))),
        _ => lines.push(Line::from("    (empty)")),
    }

    let detail = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(detail, popup_area);
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
