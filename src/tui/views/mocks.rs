use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
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
        let enabled_style = if m.enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Gray) };
        let style = if i == app.mock_selected {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let address = app.ports.iter()
            .find(|p| p.id == m.port_id)
            .map(|p| format!("{}:{}", app.system_ip, p.port))
            .unwrap_or_else(|| m.port_id.to_string());
        Row::new(vec![
            Cell::from(m.id.to_string()),
            Cell::from(address),
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
            Constraint::Length(22),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Min(15),
            Constraint::Length(4),
        ],
    )
    .header(
        Row::new(vec!["ID", "Address", "Method", "Path", "Name", "On"])
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
        Span::styled("Space", Style::default().fg(Color::Cyan)), Span::raw(": toggle on/off  "),
        Span::styled("F1", Style::default().fg(Color::Magenta)), Span::raw(": built-in functions "),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

const MOCK_LABELS: &[&str] = &[
    "Port ID *", "Method *", "Path *", "Name *", "Description",
    "Request Params", "Response Status *", "Delay (ms) *", "Response Headers", "Body Source *",
    "Response Body / File Path", "Pagination",
    "Page param", "Page size param", "Data field", "Total field",
];

pub fn draw_modal(f: &mut Frame, app: &mut App) {
    use ratatui::widgets::Clear;
    use crate::tui::app::{
        ModalKind, BODY_FIELD_IDX, BODY_SOURCE_FIELD_IDX,
        HEADER_FIELD_IDX, METHOD_FIELD_IDX, PAGINATION_ENABLED_FIELD_IDX,
        PAGINATION_PAGE_PARAM_FIELD_IDX, PAGINATION_SIZE_PARAM_FIELD_IDX,
        PAGINATION_DATA_FIELD_IDX, PAGINATION_TOTAL_FIELD_IDX,
        PATH_FIELD_IDX, PORT_ID_FIELD_IDX, REQUEST_PARAMS_FIELD_IDX,
    };

    fn chips_or_placeholder<'a>(raw: &'a str, placeholder: &'a str) -> Line<'a> {
        let chips: Vec<&str> = raw.split('|').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
        if chips.is_empty() {
            Line::from(Span::styled(placeholder, Style::default().fg(Color::DarkGray)))
        } else {
            Line::from(chips.iter().map(|p| {
                Span::styled(format!("[{}]  ", p), Style::default().fg(Color::Cyan))
            }).collect::<Vec<_>>())
        }
    }

    let is_edit = matches!(app.modal, Some(ModalKind::MockEdit));

    let body_source = app.modal_fields.get(BODY_SOURCE_FIELD_IDX)
        .map(|s| s.as_str())
        .unwrap_or("Inline");
    let body_inline = body_source == "Inline";
    let body_field_h: u16 = if body_inline { 12 } else { 3 };

    let pagination_on = app.pagination_on();
    let method = app.modal_fields.get(METHOD_FIELD_IDX).map(|s| s.as_str()).unwrap_or("GET");
    let hide_params = method == "PUT" || method == "DELETE";

    let all_visible: Vec<usize> = (0..MOCK_LABELS.len())
        .filter(|&i| {
            if i == REQUEST_PARAMS_FIELD_IDX && hide_params { return false; }
            if matches!(i, PAGINATION_PAGE_PARAM_FIELD_IDX..=PAGINATION_TOTAL_FIELD_IDX) && !pagination_on { return false; }
            true
        })
        .collect();

    let field_h = |i: usize| -> u16 {
        if i == BODY_FIELD_IDX && body_inline { body_field_h } else { 3 }
    };

    // Cap modal height: leave 4 rows for tab bar + frame margins
    let screen_h = f.area().height;
    let max_modal_h = screen_h.saturating_sub(4).max(10);
    // Inner available for fields: modal - 2 border - 1 hint
    let available: u16 = max_modal_h.saturating_sub(3);

    // Auto-adjust modal_scroll so the active field stays in view
    let active_pos = all_visible.iter().position(|&i| i == app.modal_field_idx).unwrap_or(0);
    if app.modal_scroll > active_pos {
        app.modal_scroll = active_pos;
    }
    'fwd: loop {
        let mut h = 0u16;
        let mut last = app.modal_scroll;
        for pos in app.modal_scroll..all_visible.len() {
            let fh = field_h(all_visible[pos]);
            if h + fh > available { break; }
            h += fh;
            last = pos;
        }
        if active_pos <= last { break 'fwd; }
        if app.modal_scroll + 1 >= all_visible.len() { break 'fwd; }
        app.modal_scroll += 1;
    }

    // Collect render_fields (those that fit within available height)
    let mut render_fields: Vec<usize> = Vec::new();
    let mut total_h = 0u16;
    for pos in app.modal_scroll..all_visible.len() {
        let fi = all_visible[pos];
        let fh = field_h(fi);
        if total_h + fh > available { break; }
        render_fields.push(fi);
        total_h += fh;
    }

    let has_above = app.modal_scroll > 0;
    let last_rendered = render_fields.last().copied().unwrap_or(0);
    let last_visible_idx = *all_visible.last().unwrap_or(&0);
    let has_below = last_rendered < last_visible_idx;

    let base_title = if is_edit { " Edit Mock " } else { " New Mock " };
    let title = match (has_above, has_below) {
        (true,  true)  => format!("{}▲▼", base_title),
        (true,  false) => format!("{}▲", base_title),
        (false, true)  => format!("{}▼", base_title),
        (false, false) => base_title.to_owned(),
    };

    // Modal height = fields that fit + 2 border + 1 hint (use absolute pixels, not %, to avoid rounding clipping)
    let modal_h = (total_h + 3).min(max_modal_h);
    let screen = f.area();
    let modal_w = (screen.width * 65 / 100).max(20);
    let area = Rect {
        x: screen.x + (screen.width.saturating_sub(modal_w)) / 2,
        y: screen.y + (screen.height.saturating_sub(modal_h)) / 2,
        width: modal_w,
        height: modal_h,
    };
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let constraints: Vec<Constraint> = render_fields.iter()
        .map(|&fi| Constraint::Length(field_h(fi)))
        .chain(std::iter::once(Constraint::Min(1)))
        .collect();

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(area);

    for (slot, &fi) in render_fields.iter().enumerate() {
        let label = MOCK_LABELS[fi];
        let raw = app.modal_fields.get(fi).map(|s| s.as_str()).unwrap_or("");

        let is_select = fi == PORT_ID_FIELD_IDX
            || fi == METHOD_FIELD_IDX
            || fi == BODY_SOURCE_FIELD_IDX
            || fi == PAGINATION_ENABLED_FIELD_IDX;

        let display = if fi == PORT_ID_FIELD_IDX {
            let port_id: i64 = raw.parse().unwrap_or(0);
            if let Some(p) = app.ports.iter().find(|p| p.id == port_id) {
                format!("◀ :{} {} ▶", p.port, p.label)
            } else {
                format!("◀ {} ▶", raw)
            }
        } else if is_select {
            format!("◀ {} ▶", raw)
        } else {
            raw.to_owned()
        };

        let is_active = app.modal_field_idx == fi;
        let border_style = if is_active { Style::default().fg(Color::Cyan) } else { Style::default() };

        let effective_label: String = if fi == BODY_FIELD_IDX {
            if !body_inline {
                "File Path (json/txt) *".to_owned()
            } else {
                let lines = raw.chars().filter(|&c| c == '\n').count() + 1;
                format!("Response Body  [{} lines]  ↑/↓:scroll  Ctrl+U:clear", lines)
            }
        } else {
            label.to_string()
        };

        let is_multiline_body = fi == BODY_FIELD_IDX && body_inline;
        let widget = if is_active && !is_select {
            if is_multiline_body {
                let chars: Vec<char> = display.chars().collect();
                let cur = app.modal_cursor_pos.min(chars.len());
                let before: String = chars[..cur].iter().collect();
                let cursor_ch = chars.get(cur).copied().unwrap_or(' ');
                let after: String = chars[cur.saturating_add(1).min(chars.len())..].iter().collect();
                let full = format!("{}\x00{}{}", before, cursor_ch, after);
                let lines: Vec<Line> = full.split('\n').map(|line| {
                    if let Some(pos) = line.find('\x00') {
                        let b = &line[..pos];
                        let c = line[pos+1..].chars().next().unwrap_or(' ');
                        let a = &line[pos+1..].chars().skip(1).collect::<String>();
                        Line::from(vec![
                            Span::raw(b.to_owned()),
                            Span::styled(c.to_string(), Style::default().add_modifier(Modifier::REVERSED)),
                            Span::raw(a.clone()),
                        ])
                    } else {
                        Line::from(line.to_owned())
                    }
                }).collect();
                Paragraph::new(lines)
                    .wrap(Wrap { trim: false })
                    .scroll((app.modal_body_scroll as u16, 0))
                    .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
            } else {
                let chars: Vec<char> = display.chars().collect();
                let cur = app.modal_cursor_pos.min(chars.len());
                let before: String = chars[..cur].iter().collect();
                let cursor_ch = chars.get(cur).copied().unwrap_or(' ');
                let after: String = chars[cur.saturating_add(1).min(chars.len())..].iter().collect();
                Paragraph::new(Line::from(vec![
                    Span::raw(before),
                    Span::styled(cursor_ch.to_string(), Style::default().add_modifier(Modifier::REVERSED)),
                    Span::raw(after),
                ]))
                .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
            }
        } else if is_multiline_body {
            let lines: Vec<Line> = display.split('\n').map(|l| Line::from(l.to_owned())).collect();
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((app.modal_body_scroll as u16, 0))
                .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
        } else if fi == REQUEST_PARAMS_FIELD_IDX && !is_active {
            Paragraph::new(chips_or_placeholder(raw, "按 + 添加参数名，多个参数用 | 分隔"))
                .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
        } else if fi == HEADER_FIELD_IDX && !is_active {
            Paragraph::new(chips_or_placeholder(raw, "按 + 添加响应头，格式 Key: Value"))
                .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
        } else {
            Paragraph::new(Line::from(display))
                .block(Block::default().title(effective_label.as_str()).borders(Borders::ALL).border_style(border_style))
        };
        f.render_widget(widget, inner[slot]);
    }

    let (hint_text, hint_style) = if let Some(err) = &app.modal_error {
        (err.clone(), Style::default().fg(Color::Red))
    } else if app.cancel_confirm_pending {
        ("Discard changes?  Enter: yes  Esc: no".to_owned(), Style::default().fg(Color::Yellow))
    } else if app.modal_field_idx == BODY_SOURCE_FIELD_IDX {
        ("←/→: select  Tab: next  Shift+Tab: back".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PORT_ID_FIELD_IDX || app.modal_field_idx == METHOD_FIELD_IDX {
        ("←/→: select  Tab: next  Shift+Tab: back  Enter: save  Esc: cancel".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == BODY_FIELD_IDX && body_inline {
        ("↑/↓: scroll body  Ctrl+U: clear  Tab: next  Shift+Tab: back  Enter: save  Esc: cancel".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == BODY_FIELD_IDX && !body_inline {
        ("Enter full file path (e.g. /home/user/data.json)  Tab: next  Shift+Tab: back  Enter: save  Esc: cancel".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PATH_FIELD_IDX {
        ("Use {param} for path params (e.g. /users/{id})  Tab: next  Shift+Tab: back  Esc: cancel".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == REQUEST_PARAMS_FIELD_IDX {
        ("+: 新增参数  Tab: 下一项  Shift+Tab: 上一项  Enter: 保存 — 添加后，JSON 响应将按参数值过滤（如 ?name=john），非 JSON 不受影响".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PAGINATION_PAGE_PARAM_FIELD_IDX {
        ("页码参数名，留空默认 page  Tab: 下一项  Shift+Tab: 上一项  Enter: 保存".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PAGINATION_SIZE_PARAM_FIELD_IDX {
        ("页大小参数名，留空默认 pageSize（未传时每页 10 条）  Tab: 下一项  Enter: 保存".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PAGINATION_DATA_FIELD_IDX {
        ("数组字段路径，支持点号路径（如 body.list），留空表示顶层数组  Tab: 下一项  Enter: 保存".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == PAGINATION_TOTAL_FIELD_IDX {
        ("回写总条数字段路径，支持点号路径（如 body.total），留空则跳过  Tab: 下一项  Enter: 保存".to_owned(), Style::default().fg(Color::DarkGray))
    } else if app.modal_field_idx == HEADER_FIELD_IDX {
        let text = if let Some(sug) = app.header_autocomplete_suggestion() {
            format!("+: 新增  →: \"{}\"  格式 Key: Value，多个用 |  Tab: 下一项  Shift+Tab: 上一项  Enter: 保存", sug)
        } else {
            "+: 新增响应头  格式 Key: Value，多个用 |  Tab: 下一项  Shift+Tab: 上一项  Enter: 保存".to_owned()
        };
        (text, Style::default().fg(Color::DarkGray))
    } else {
        ("Tab: next  Shift+Tab: back  Enter: save  Esc: cancel".to_owned(), Style::default().fg(Color::DarkGray))
    };
    let hint = Paragraph::new(hint_text.as_str()).style(hint_style);
    if let Some(last) = inner.last() {
        f.render_widget(hint, *last);
    }
}
