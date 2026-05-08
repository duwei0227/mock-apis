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
    let help = Paragraph::new(Line::from(vec![
        Span::raw(" ↑/↓: scroll  "),
        Span::styled("r", Style::default().fg(Color::Yellow)), Span::raw(": requests  "),
        Span::styled("s", Style::default().fg(Color::Yellow)), Span::raw(": system  "),
        Span::styled("c", Style::default().fg(Color::Red)), Span::raw(": clear  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)), Span::raw(": detail  "),
        Span::styled("Esc", Style::default().fg(Color::DarkGray)), Span::raw(": close detail"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);

    // ---- detail overlay ----
    if app.log_detail_open && app.log_tab == LogTab::Request {
        let idx = app.request_log_state.selected().unwrap_or(0);
        if let Some(log) = app.request_logs.get(idx) {
            draw_request_detail(f, log, area, app.log_detail_scroll);
        }
    }
}

fn draw_request_logs(f: &mut Frame, app: &App, area: Rect) {
    // 2 border lines + 1 header + 1 header underline margin = 4 overhead lines
    let visible = area.height.saturating_sub(4) as usize;
    let sel     = app.request_log_state.selected().unwrap_or(0);

    let rows: Vec<Row> = app.request_logs
        .iter()
        .enumerate()
        .take(visible + 1)
        .map(|(abs_i, r)| {
            let status_style = if r.response_status < 400 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let row_style = if abs_i == sel {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
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
            ])
            .style(row_style)
        })
        .collect();

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
    let visible = area.height.saturating_sub(4) as usize;
    let sel     = app.system_log_state.selected().unwrap_or(0);

    let rows: Vec<Row> = app.system_logs
        .iter()
        .enumerate()
        .take(visible + 1)
        .map(|(abs_i, s)| {
            let level_style = match s.level.as_str() {
                "ERROR" => Style::default().fg(Color::Red),
                "WARN"  => Style::default().fg(Color::Yellow),
                "INFO"  => Style::default().fg(Color::Cyan),
                _       => Style::default().fg(Color::DarkGray),
            };
            let row_style = if abs_i == sel {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(s.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::from(Span::styled(s.level.clone(), level_style)),
                Cell::from(s.target.clone()),
                Cell::from(s.message.clone()),
            ])
            .style(row_style)
        })
        .collect();

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

fn draw_request_detail(f: &mut Frame, log: &crate::models::RequestLog, area: Rect, scroll: usize) {
    let popup_area = centered_rect(80, 85, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Request Detail  ↑/↓: scroll  Esc: close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Styles
    let label_style    = Style::default().add_modifier(Modifier::BOLD);
    let section_style  = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let hdr_key_style  = Style::default().fg(Color::Yellow);
    let dim_style      = Style::default().fg(Color::DarkGray);

    let mut lines: Vec<Line> = Vec::new();

    // ── Request ──
    lines.push(Line::from(Span::styled("── Request ──────────────────────────", Style::default().fg(Color::Yellow))));
    lines.push(Line::from(vec![
        Span::styled("  Method: ", label_style),
        Span::raw(log.method.clone()),
        Span::raw("  "),
        Span::styled("Path: ", label_style),
        Span::raw(log.path.clone()),
    ]));
    if let Some(qs) = &log.query_string {
        lines.push(Line::from(vec![
            Span::styled("  Query: ", label_style),
            Span::raw(qs.clone()),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("  Client IP: ", label_style),
        Span::raw(log.client_ip.as_deref().unwrap_or("-").to_owned()),
        Span::raw("  "),
        Span::styled("Port: ", label_style),
        Span::raw(log.port.to_string()),
        Span::raw("  "),
        Span::styled("Time: ", label_style),
        Span::raw(log.created_at.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Request Headers", section_style)));
    if log.request_headers.is_empty() {
        lines.push(Line::from(Span::styled("    (none)", dim_style)));
    } else {
        let mut sorted: Vec<_> = log.request_headers.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in sorted {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{}: ", k), hdr_key_style),
                Span::raw(v.clone()),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Request Body", section_style)));
    match &log.request_body {
        Some(b) if !b.is_empty() => lines.push(Line::from(format!("    {}", b))),
        _ => lines.push(Line::from(Span::styled("    (empty)", dim_style))),
    }

    // ── Response ──
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("── Response ─────────────────────────", Style::default().fg(Color::Green))));
    let status_style = if log.response_status < 400 {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };
    lines.push(Line::from(vec![
        Span::styled("  Status: ", label_style),
        Span::styled(log.response_status.to_string(), status_style),
        Span::raw("  "),
        Span::styled("Duration: ", label_style),
        Span::raw(format!("{}ms", log.duration_ms)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Response Headers", section_style)));
    if log.response_headers.is_empty() {
        lines.push(Line::from(Span::styled("    (none)", dim_style)));
    } else {
        let mut sorted: Vec<_> = log.response_headers.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in sorted {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{}: ", k), hdr_key_style),
                Span::raw(v.clone()),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Response Body", section_style)));
    match &log.response_body {
        Some(b) if !b.is_empty() => lines.push(Line::from(format!("    {}", b))),
        _ => lines.push(Line::from(Span::styled("    (empty)", dim_style))),
    }

    let detail = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
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
