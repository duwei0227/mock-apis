pub mod app;
pub mod event;
pub mod views;

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use tokio::sync::mpsc;

use crate::error::Result;
use crate::traits::LogQuery;
use crate::AppState;

use app::{App, ConfirmAction, ModalKind, Tab, BODY_SOURCE_FIELD_IDX, HEADER_FIELD_IDX, METHOD_FIELD_IDX, PORT_ID_FIELD_IDX};
use event::{spawn_event_task, Event};

pub async fn run(state: AppState) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, state).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableBracketedPaste, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: AppState,
) -> Result<()> {
    let (ev_tx, mut ev_rx) = mpsc::channel(256);
    let log_rx = state.log_tx.subscribe();
    spawn_event_task(log_rx, ev_tx);

    let mut app = App::new(state.clone());

    // Initial data load.
    refresh_ports(&mut app).await;
    refresh_mocks(&mut app).await;
    load_initial_logs(&mut app).await;

    loop {
        terminal.draw(|f| render(f, &app))?;

        let Some(ev) = ev_rx.recv().await else { break };

        match ev {
            Event::Log(log_ev) => {
                app.push_log_event(log_ev);
            }
            Event::Paste(text) => {
                if app.modal.is_some() {
                    app.modal_paste(&text);
                }
            }
            Event::Tick => {}
            Event::Resize => {}
            Event::Key(key) => {
                use crossterm::event::{KeyCode, KeyModifiers};
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                // Global quit.
                if (key.code == KeyCode::Char('q') && !app.modal.is_some() && !app.show_fn_help)
                    || (ctrl && key.code == KeyCode::Char('c'))
                {
                    break;
                }

                // Toggle template function help with '?'.
                if key.code == KeyCode::Char('?') {
                    app.show_fn_help = !app.show_fn_help;
                    continue;
                }
                if app.show_fn_help {
                    if key.code == KeyCode::Esc {
                        app.show_fn_help = false;
                    }
                    continue;
                }

                if app.modal.is_some() {
                    handle_modal_key(&mut app, key.code, &state).await;
                } else {
                    handle_normal_key(&mut app, key.code, &state).await;
                }
            }
        }
    }
    Ok(())
}

async fn handle_normal_key(
    app: &mut App,
    code: crossterm::event::KeyCode,
    state: &AppState,
) {
    use crossterm::event::KeyCode;
    match app.active_tab {
        Tab::Ports => match code {
            KeyCode::Char('1') => app.active_tab = Tab::Ports,
            KeyCode::Char('2') => app.active_tab = Tab::Mocks,
            KeyCode::Char('3') => app.active_tab = Tab::Logs,
            KeyCode::Char('4') => app.active_tab = Tab::Functions,
            KeyCode::Tab       => app.active_tab = app.active_tab.next(),
            KeyCode::Down | KeyCode::Char('j') => app.port_list_nav_down(),
            KeyCode::Up   | KeyCode::Char('k') => app.port_list_nav_up(),
            KeyCode::Char('n') => app.open_port_create(),
            KeyCode::Char('e') => app.open_port_edit(),
            KeyCode::Char('d') => {
                if let Some((port, id)) = app.selected_port().map(|p| (p.port, p.id)) {
                    app.confirm_message = format!("Delete port {}?", port);
                    app.confirm_action = Some(ConfirmAction::DeletePort(id));
                    app.modal = Some(ModalKind::Confirm);
                }
            }
            KeyCode::Char(' ') => {
                if let Some(p) = app.selected_port().cloned() {
                    if app.running_port_ids.contains(&p.id) {
                        let _ = state.port_manager.stop_port(p.id).await;
                        app.running_port_ids.retain(|&id| id != p.id);
                    } else {
                        let _ = state.port_manager.start_port(p.id).await;
                        app.running_port_ids.push(p.id);
                    }
                }
            }
            _ => {}
        },
        Tab::Mocks => match code {
            KeyCode::Char('1') => app.active_tab = Tab::Ports,
            KeyCode::Char('2') => app.active_tab = Tab::Mocks,
            KeyCode::Char('3') => app.active_tab = Tab::Logs,
            KeyCode::Char('4') => app.active_tab = Tab::Functions,
            KeyCode::Tab       => app.active_tab = app.active_tab.next(),
            KeyCode::Down | KeyCode::Char('j') => app.mock_list_nav_down(),
            KeyCode::Up   | KeyCode::Char('k') => app.mock_list_nav_up(),
            KeyCode::Char('n') => app.open_mock_create(),
            KeyCode::Char('e') => app.open_mock_edit(),
            KeyCode::Char('d') => {
                if let Some((name, id)) = app.selected_mock().map(|m| (m.name.clone(), m.id)) {
                    app.confirm_message = format!("Delete mock \"{}\"?", name);
                    app.confirm_action = Some(ConfirmAction::DeleteMock(id));
                    app.modal = Some(ModalKind::Confirm);
                }
            }
            KeyCode::Char(' ') => {
                if let Some(m) = app.selected_mock().cloned() {
                    let new_state = !m.enabled;
                    let _ = state.mock_store.set_mock_enabled(m.id, new_state).await;
                    let _ = state.port_manager.restart_port(m.port_id).await;
                    refresh_mocks(app).await;
                }
            }
            _ => {}
        },
        Tab::Logs => match code {
            KeyCode::Char('1') => app.active_tab = Tab::Ports,
            KeyCode::Char('2') => app.active_tab = Tab::Mocks,
            KeyCode::Char('3') => app.active_tab = Tab::Logs,
            KeyCode::Char('4') => app.active_tab = Tab::Functions,
            KeyCode::Tab       => app.active_tab = app.active_tab.next(),
            KeyCode::Esc       => { app.log_detail_open = false; }
            KeyCode::Enter => {
                if app.log_tab == crate::tui::app::LogTab::Request {
                    app.log_follow = false;
                    app.log_detail_open = !app.log_detail_open;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => { app.log_follow = false; app.log_nav_down(); }
            KeyCode::Up   | KeyCode::Char('k') => { app.log_follow = false; app.log_nav_up(); }
            KeyCode::Char('f') => { app.log_follow = !app.log_follow; if app.log_follow { app.log_detail_open = false; } }
            KeyCode::Char('r') => { app.log_tab = crate::tui::app::LogTab::Request; app.log_detail_open = false; }
            KeyCode::Char('s') => { app.log_tab = crate::tui::app::LogTab::System; app.log_detail_open = false; }
            _ => {}
        },
        Tab::Functions => match code {
            KeyCode::Char('1') => app.active_tab = Tab::Ports,
            KeyCode::Char('2') => app.active_tab = Tab::Mocks,
            KeyCode::Char('3') => app.active_tab = Tab::Logs,
            KeyCode::Char('4') => app.active_tab = Tab::Functions,
            KeyCode::Tab       => app.active_tab = app.active_tab.next(),
            _ => {}
        },
    }
}

async fn handle_modal_key(
    app: &mut App,
    code: crossterm::event::KeyCode,
    state: &AppState,
) {
    use crossterm::event::KeyCode;

    // If user pressed Esc and we're waiting for their confirmation to discard:
    if app.cancel_confirm_pending {
        match code {
            KeyCode::Enter => { app.dismiss_modal(); }
            KeyCode::Esc   => { app.cancel_confirm_pending = false; }
            _ => {}
        }
        return;
    }

    // Clear any previous validation error on each keypress.
    app.modal_error = None;

    let is_mock_modal      = matches!(app.modal, Some(ModalKind::MockCreate) | Some(ModalKind::MockEdit));
    let on_port_field      = app.modal_field_idx == PORT_ID_FIELD_IDX;
    let on_method_field    = app.modal_field_idx == METHOD_FIELD_IDX;
    let on_header_field    = app.modal_field_idx == HEADER_FIELD_IDX;
    let on_body_src_field  = app.modal_field_idx == BODY_SOURCE_FIELD_IDX;
    let on_select_field    = is_mock_modal && (on_port_field || on_method_field || on_body_src_field);

    match code {
        KeyCode::Esc if matches!(app.modal, Some(ModalKind::Confirm)) => app.dismiss_modal(),
        KeyCode::Esc => { app.cancel_confirm_pending = true; }
        KeyCode::Tab => app.modal_field_next(),
        KeyCode::BackTab => app.modal_field_prev(),
        KeyCode::Left if is_mock_modal && on_port_field       => app.cycle_port_field(false),
        KeyCode::Right if is_mock_modal && on_port_field      => app.cycle_port_field(true),
        KeyCode::Left if is_mock_modal && on_method_field     => app.cycle_method_field(false),
        KeyCode::Right if is_mock_modal && on_method_field    => app.cycle_method_field(true),
        KeyCode::Left if is_mock_modal && on_body_src_field   => app.cycle_body_source_field(false),
        KeyCode::Right if is_mock_modal && on_body_src_field  => app.cycle_body_source_field(true),
        KeyCode::Right if is_mock_modal && on_header_field
            && app.header_autocomplete_suggestion().is_some() => app.accept_header_autocomplete(),
        KeyCode::Left  if !on_select_field => app.modal_cursor_left(),
        KeyCode::Right if !on_select_field => app.modal_cursor_right(),
        KeyCode::Backspace if !on_select_field => app.modal_backspace(),
        KeyCode::Char(c) if !on_select_field => app.modal_type_char(c),
        KeyCode::Enter => {
            match app.modal.clone() {
                Some(ModalKind::PortCreate) => {
                    if let Some(err) = app.validate_port_modal() {
                        app.modal_error = Some(err);
                    } else {
                        let port: u16 = app.modal_fields.get(0).and_then(|s| s.parse().ok()).unwrap_or(8080);
                        let label = app.modal_fields.get(1).cloned().unwrap_or_default();
                        match state.port_store.create_port(port, &label).await {
                            Ok(_) => {
                                app.status_msg = None;
                                app.dismiss_modal();
                                refresh_ports(app).await;
                            }
                            Err(_) => {
                                app.modal_error = Some(format!("Port {} is already in use", port));
                            }
                        }
                    }
                }
                Some(ModalKind::PortEdit) => {
                    if let Some(err) = app.validate_port_modal() {
                        app.modal_error = Some(err);
                    } else if let Some(p) = app.selected_port().cloned() {
                        let label = app.modal_fields.get(1).cloned().unwrap_or_default();
                        let enabled = p.enabled;
                        if let Ok(_) = state.port_store.update_port(p.id, &label, enabled).await {
                            app.dismiss_modal();
                            refresh_ports(app).await;
                        }
                    }
                }
                Some(ModalKind::MockCreate) => {
                    if let Some(err) = app.validate_mock_modal() {
                        app.modal_error = Some(err);
                    } else if let Some(req) = app.build_create_mock() {
                        if let Ok(m) = state.mock_store.create_mock(req).await {
                            let _ = state.port_manager.restart_port(m.port_id).await;
                            app.dismiss_modal();
                            refresh_mocks(app).await;
                        }
                    }
                }
                Some(ModalKind::MockEdit) => {
                    if let Some(err) = app.validate_mock_modal() {
                        app.modal_error = Some(err);
                    } else if let Some(mock_id) = app.selected_mock().map(|m| m.id) {
                        let port_id = app.selected_mock().map(|m| m.port_id).unwrap_or(0);
                        let req = app.build_update_mock();
                        if let Ok(_) = state.mock_store.update_mock(mock_id, req).await {
                            let _ = state.port_manager.restart_port(port_id).await;
                            app.dismiss_modal();
                            refresh_mocks(app).await;
                        }
                    }
                }
                Some(ModalKind::Confirm) => {
                    if let Some(action) = app.confirm_action.clone() {
                        match action {
                            ConfirmAction::DeletePort(id) => {
                                let _ = state.port_manager.stop_port(id).await;
                                let _ = state.port_store.delete_port(id).await;
                                refresh_ports(app).await;
                            }
                            ConfirmAction::DeleteMock(id) => {
                                if let Some(m) = state.mock_store.get_mock(id).await.ok().flatten() {
                                    let _ = state.mock_store.delete_mock(id).await;
                                    let _ = state.port_manager.restart_port(m.port_id).await;
                                }
                                refresh_mocks(app).await;
                            }
                        }
                    }
                    app.dismiss_modal();
                }
                _ => {}
            }
        }
        _ => {}
    }
}

async fn refresh_ports(app: &mut App) {
    if let Ok(ports) = app.state.port_store.list_ports().await {
        app.ports = ports;
        app.running_port_ids = app.state.port_manager.running_ports().await;
        app.port_selected = app.port_selected.min(app.ports.len().saturating_sub(1));
    }
}

async fn refresh_mocks(app: &mut App) {
    if let Ok(mocks) = app.state.mock_store.list_mocks(None).await {
        app.mocks = mocks;
        app.mock_selected = app.mock_selected.min(app.mocks.len().saturating_sub(1));
    }
}

async fn load_initial_logs(app: &mut App) {
    let query = LogQuery { page_size: 100, ..Default::default() };
    if let Ok(page) = app.state.log_store.list_request_logs(query.clone()).await {
        app.request_logs = page.items;
    }
    if let Ok(page) = app.state.log_store.list_system_logs(query).await {
        app.system_logs = page.items;
    }
}

fn render(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // ---- top tab bar ----
    let titles: Vec<Line> = vec![
        Line::from(" [1] Ports  "),
        Line::from(" [2] Mocks  "),
        Line::from(" [3] Logs  "),
        Line::from(" [4] Functions  "),
    ];
    let active = match app.active_tab {
        Tab::Ports     => 0,
        Tab::Mocks     => 1,
        Tab::Logs      => 2,
        Tab::Functions => 3,
    };
    let tabs = Tabs::new(titles)
        .select(active)
        .block(
            Block::default()
                .title(" mock-apis  q:quit ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // ---- tab content ----
    match app.active_tab {
        Tab::Ports     => views::ports::draw(f, app, chunks[1]),
        Tab::Mocks     => views::mocks::draw(f, app, chunks[1]),
        Tab::Logs      => views::logs::draw(f, app, chunks[1]),
        Tab::Functions => views::functions::draw(f, app, chunks[1]),
    }

    // ---- modal overlay ----
    match app.modal {
        Some(ModalKind::PortCreate) | Some(ModalKind::PortEdit) => {
            views::ports::draw_modal(f, app);
        }
        Some(ModalKind::MockCreate) | Some(ModalKind::MockEdit) => {
            views::mocks::draw_modal(f, app);
        }
        Some(ModalKind::Confirm) => draw_confirm(f, app),
        None => {}
    }

    // ---- template function help overlay ----
    if app.show_fn_help {
        views::fn_help::draw(f, app);
    }

    // ---- status bar ----
    if let Some(msg) = &app.status_msg {
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow));
        let status_area = ratatui::layout::Rect {
            x: 0,
            y: area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };
        f.render_widget(status, status_area);
    }
}

fn draw_confirm(f: &mut ratatui::Frame, app: &App) {
    use ratatui::widgets::Clear;
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let text = format!("{}\n\nEnter: confirm   Esc: cancel", app.confirm_message);
    let widget = Paragraph::new(text).block(block);
    f.render_widget(widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
