use crate::core::EventStatus;
use crate::tui::app::{App, InputMode, Mode};
use chrono::Datelike;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};
use std::time::Duration;

const ACCENT: Color = Color::Red;
const ORANGE: Color = Color::Rgb(255, 165, 0);

pub fn run(store: crate::core::EventStore, store_path: std::path::PathBuf) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use std::io;

    let mut app = App::new(store, store_path);

    // Check if we're in a TTY
    if !atty::is(atty::Stream::Stdout) {
        anyhow::bail!("TUI requires a terminal (TTY). Use CLI commands instead.");
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        app.clear_old_message();
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    Mode::CategoryList => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.mode = Mode::CalendarView;
                            app.refresh_events();
                        }
                        KeyCode::Char('a') => {
                            app.mode = Mode::AddCategory;
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('x') => {
                            if let Err(e) = app.delete_event() {
                                app.set_message(format!("Error: {}", e));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next_category(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_category(),
                        _ => {}
                    },
                    Mode::EventDetail => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.mode = Mode::CalendarView;
                        }
                        KeyCode::Char('d') => {
                            if let Err(e) = app.toggle_selected_day_event_complete() {
                                app.set_message(format!("Error: {}", e));
                            }
                        }
                        _ => {}
                    },
                    Mode::CalendarView => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => {
                            app.mode = Mode::AddEvent;
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('t') => {
                            app.go_to_today();
                            app.set_message("Jumped to today".to_string());
                        }
                        KeyCode::Char('d') => {
                            if let Err(e) = app.toggle_selected_day_event_complete() {
                                app.set_message(format!("Error: {}", e));
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Err(e) = app.delete_selected_day_event() {
                                app.set_message(format!("Error: {}", e));
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Tab => {
                            app.next_day_event();
                        }
                        KeyCode::Char('k') | KeyCode::BackTab => {
                            app.previous_day_event();
                        }
                        KeyCode::Enter => {
                            app.view_selected_day_event_details();
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        KeyCode::Up => {
                            app.move_cursor_up();
                        }
                        KeyCode::Down => {
                            app.move_cursor_down();
                        }
                        _ => {}
                    },
                    Mode::AddEvent => match app.input_mode {
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                if !app.input.is_empty() {
                                    let title = app.input.clone();
                                    if let Err(e) = app.add_event(title) {
                                        app.set_message(format!("Error: {}", e));
                                    }
                                }
                                app.mode = Mode::CalendarView;
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => app.input.push(c),
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.mode = Mode::CalendarView;
                                app.input_mode = InputMode::Normal;
                                app.input.clear();
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    Mode::AddCategory => match app.input_mode {
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                if !app.input.is_empty() {
                                    let name = app.input.clone();
                                    if let Err(e) = app.add_category(name) {
                                        app.set_message(format!("Error: {}", e));
                                    }
                                }
                                app.mode = Mode::CategoryList;
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => app.input.push(c),
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.mode = Mode::CategoryList;
                                app.input_mode = InputMode::Normal;
                                app.input.clear();
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let area = f.size();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(area);

    render_sidebar(f, app, chunks[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Min(0),    // content
            Constraint::Length(3), // status bar
        ])
        .split(chunks[1]);

    render_titlebar(f, app, right[0]);

    match app.mode {
        Mode::CategoryList => render_category_list(f, app, right[1]),
        Mode::EventDetail => render_event_detail(f, app, right[1]),
        Mode::CalendarView => render_calendar(f, app, right[1]),
        Mode::AddEvent => {
            render_calendar(f, app, right[1]);
            render_input_popup(f, app, right[1], "New Event", "Title");
        }
        Mode::AddCategory => {
            render_category_list(f, app, right[1]);
            render_input_popup(f, app, right[1], "New Category", "Name");
        }
    }

    render_statusbar(f, app, right[2]);
}

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.store.get_stats();

    let mut lines = vec![
        Line::from(Span::styled(" Stats", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        stat_line("📅", Color::Blue, format!("Total      {}", stats.total_events)),
        stat_line("🔵", Color::Blue, format!("Scheduled  {}", stats.scheduled)),
        stat_line("🟡", Color::Yellow, format!("In Progress {}", stats.in_progress)),
        stat_line("🟢", Color::Green, format!("Completed  {}", stats.completed)),
    ];

    if stats.overdue > 0 {
        lines.push(stat_line("✗", Color::Red, format!("Overdue    {}", stats.overdue)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Categories", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));

    for category in app.store.list_categories().iter().take(6) {
        let name = truncate(&category.name, 14);
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(name, Style::default().fg(Color::White)),
        ]));
    }

    if app.store.list_categories().is_empty() {
        lines.push(Line::from(Span::styled("  none", Style::default().fg(Color::DarkGray))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn stat_line(icon: &str, color: Color, label: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {} ", icon), Style::default().fg(color)),
        Span::styled(label, Style::default().fg(Color::White)),
    ])
}

fn render_titlebar(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        Mode::AddEvent => " FireCalendar",
        Mode::CategoryList | Mode::AddCategory => " Categories",
        Mode::EventDetail => " Event Detail",
        Mode::CalendarView => " FireCalendar",
    };

    let mut spans = vec![
        Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ];

    if let Some(s) = &app.filter_status {
        let label = match s {
            EventStatus::Scheduled => "scheduled",
            EventStatus::InProgress => "in-progress",
            EventStatus::Completed => "completed",
            EventStatus::Cancelled => "cancelled",
        };
        spans.push(Span::styled(format!("  [{}]", label), Style::default().fg(Color::Yellow)));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let p = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(p, area);
}

fn render_category_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    if app.categories.is_empty() {
        let p = Paragraph::new("No categories found. Press 'a' to add one.")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let style = if i == app.selected_category_index {
                Style::default().bg(DIM_ACCENT).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                category.id.to_string(),
                truncate(&category.name, 30),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec!["ID", "Name"])
        .style(Style::default().fg(Color::Gray));

    let table = Table::new(rows, [Constraint::Length(36), Constraint::Min(0)])
        .header(header)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(table, area);
}

fn render_calendar(f: &mut Frame, app: &App, area: Rect) {
    render_calendar_grid(f, app, area);
}

fn render_calendar_grid(f: &mut Frame, app: &App, area: Rect) {
    let month_names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    ];

    let month_name = if app.calendar_month >= 1 && app.calendar_month <= 12 {
        month_names[(app.calendar_month - 1) as usize]
    } else {
        "Unknown"
    };
    let title = format!(" {} {} ", month_name, app.calendar_year);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Calculate calendar grid
    let first_day = match chrono::NaiveDate::from_ymd_opt(app.calendar_year, app.calendar_month, 1) {
        Some(date) => date,
        None => {
            // Fallback to current date if invalid
            let now = chrono::Utc::now();
            match chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()) {
                Some(date) => date,
                None => {
                    // Ultimate fallback: use a safe default date
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
                }
            }
        }
    };
    let weekday_of_first = first_day.weekday().num_days_from_sunday();
    let days_in_month = app.days_in_month();

    // Weekday headers
    let weekday_headers = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    // Build calendar weeks
    let mut weeks: Vec<Vec<Option<u32>>> = Vec::new();
    let mut current_week: Vec<Option<u32>> = vec![None; 7];
    let mut day_counter = 1u32;

    // Empty cells before first day
    for i in 0..weekday_of_first {
        current_week[i as usize] = None;
    }

    // Fill first week
    for i in weekday_of_first..7 {
        if day_counter <= days_in_month {
            current_week[i as usize] = Some(day_counter);
            day_counter += 1;
        }
    }
    weeks.push(current_week);

    // Remaining weeks
    while day_counter <= days_in_month {
        current_week = vec![None; 7];
        for i in 0..7 {
            if day_counter <= days_in_month {
                current_week[i] = Some(day_counter);
                day_counter += 1;
            }
        }
        weeks.push(current_week);
    }

    // Create layout: header row + 6 week rows
    let num_weeks = weeks.len();
    let header_height = 1;
    let week_height = if num_weeks > 0 {
        (inner_area.height - header_height) / num_weeks as u16
    } else {
        (inner_area.height - header_height) / 6
    };

    let mut constraints = vec![Constraint::Length(header_height)];
    for _ in 0..num_weeks {
        constraints.push(Constraint::Min(week_height));
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice())
        .split(inner_area);

    // Render header row
    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
        .split(rows[0]);

    for (i, &day) in weekday_headers.iter().enumerate() {
        let p = Paragraph::new(day)
            .style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(p, header_cols[i]);
    }

    // Render week rows
    for (week_idx, week) in weeks.iter().enumerate() {
        let row_area = rows[week_idx + 1];
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ])
            .split(row_area);

        for (col_idx, &day) in week.iter().enumerate() {
            if let Some(day) = day {
                let events = app.get_events_for_day(day);
                let now = chrono::Utc::now();
                let is_today = now.year() == app.calendar_year && now.month() == app.calendar_month && now.day() == day;
                let is_selected = app.selected_day == Some(day);

                let day_str = format!("{:2}", day);
                let event_count = events.len();

                // Event indicator
                let event_indicator = if event_count > 0 {
                    if event_count == 1 {
                        "•".to_string()
                    } else if event_count <= 3 {
                        "•".repeat(event_count)
                    } else {
                        "•+".to_string()
                    }
                } else {
                    String::new()
                };

                let content = if event_count > 0 {
                    format!("{}\n{}", day_str, event_indicator)
                } else {
                    day_str
                };

                // Style calculation
                let border_color = if is_selected {
                    ORANGE
                } else {
                    Color::DarkGray
                };

                let day_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color));

                // Add "Today" text below day number if it's today
                let content_with_today = if is_today {
                    format!("{}\nToday", content)
                } else {
                    content
                };

                let p = Paragraph::new(content_with_today)
                    .block(day_block)
                    .alignment(Alignment::Center);
                f.render_widget(p, cols[col_idx]);
            } else {
                let day_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray));
                f.render_widget(day_block, cols[col_idx]);
            }
        }
    }
}

fn render_day_details(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Day Details ");

    if let Some(selected_day) = app.selected_day {
        let events = app.get_events_for_day(selected_day);
        let now = chrono::Utc::now();
        let is_today = now.year() == app.calendar_year && now.month() == app.calendar_month && now.day() == selected_day;

        let month_names = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];

        let month_name = if app.calendar_month >= 1 && app.calendar_month <= 12 {
            month_names[(app.calendar_month - 1) as usize]
        } else {
            "Unknown"
        };
        let date_str = format!("{} {}, {}", month_name, selected_day, app.calendar_year);

        let mut lines = vec![
            Line::from(vec![
                Span::styled("📅 ", Style::default().fg(ORANGE)),
                Span::styled(date_str, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];

        if is_today {
            lines.push(Line::from(vec![
                Span::styled("★ ", Style::default().fg(Color::Yellow)),
                Span::styled("Today", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled("Events: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", events.len()), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));

        if events.is_empty() {
            lines.push(Line::from(
                Span::styled("No events scheduled", Style::default().fg(Color::DarkGray))
            ));
        } else {
            for (i, event) in events.iter().enumerate().take(8) {
                let time_str = event.start_time.format("%H:%M").to_string();
                let status_icon = match event.status {
                    crate::core::EventStatus::Scheduled => "○",
                    crate::core::EventStatus::InProgress => "◐",
                    crate::core::EventStatus::Completed => "●",
                    crate::core::EventStatus::Cancelled => "⊘",
                };

                let status_color = match event.status {
                    crate::core::EventStatus::Scheduled => Color::Blue,
                    crate::core::EventStatus::InProgress => Color::Yellow,
                    crate::core::EventStatus::Completed => Color::Green,
                    crate::core::EventStatus::Cancelled => Color::Red,
                };

                let is_selected = i == app.selected_day_event_index;
                let base_style = if is_selected {
                    Style::default().bg(ORANGE).fg(Color::Black)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", time_str), Style::default().fg(ORANGE).patch(base_style)),
                    Span::styled(status_icon, Style::default().fg(status_color).patch(base_style)),
                    Span::styled(format!(" {}", truncate(&event.title, 20)), Style::default().fg(Color::White).patch(base_style)),
                ]));

                if let Some(location) = &event.location {
                    lines.push(Line::from(vec![
                        Span::styled("   📍 ", Style::default().fg(Color::DarkGray)),
                        Span::styled(truncate(location, 22), Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }

            if events.len() > 8 {
                lines.push(Line::from(""));
                lines.push(Line::from(
                    Span::styled(format!("... and {} more", events.len() - 8), Style::default().fg(Color::DarkGray))
                ));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No day selected")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(paragraph, area);
    }
}

fn render_event_detail(f: &mut Frame, app: &App, area: Rect) {
    // Try to get the event from the selected day first, then fall back to global list
    let event: Option<crate::core::Event> = if let Some(e) = app.get_selected_day_events().get(app.selected_day_event_index) {
        Some((*e).clone())
    } else {
        app.events.get(app.selected_event_index).cloned()
    };

    if let Some(ref event) = event {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::Gray)),
                Span::styled(&event.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:?}", event.status), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Start: ", Style::default().fg(Color::Gray)),
                Span::styled(event.start_time.format("%Y-%m-%d %H:%M UTC").to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(Color::Gray)),
                Span::styled(event.created_at.format("%Y-%m-%d %H:%M UTC").to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Updated: ", Style::default().fg(Color::Gray)),
                Span::styled(event.updated_at.format("%Y-%m-%d %H:%M UTC").to_string(), Style::default().fg(Color::White)),
            ]),
        ];

        let mut all_lines = lines;

        if let Some(end_time) = &event.end_time {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled("End: ", Style::default().fg(Color::Gray)),
                Span::styled(end_time.format("%Y-%m-%d %H:%M UTC").to_string(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(desc) = &event.description {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Gray)),
                Span::styled(desc, Style::default().fg(Color::White)),
            ]));
        }

        if let Some(location) = &event.location {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().fg(Color::Gray)),
                Span::styled(location, Style::default().fg(Color::White)),
            ]));
        }

        if !event.tags.is_empty() {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(Color::Gray)),
                Span::styled(event.tags.join(", "), Style::default().fg(Color::White)),
            ]));
        }

        let p = Paragraph::new(all_lines)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(p, area);
    }
}

fn render_input_popup(f: &mut Frame, app: &App, area: Rect, title: &str, label: &str) {
    let popup = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT));

    let _input_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(area);

    let label_paragraph = Paragraph::new(label)
        .style(Style::default().fg(Color::Gray));

    let input_paragraph = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));

    let popup_area = centered_rect(60, 20, area);

    f.render_widget(Clear, popup_area);
    f.render_widget(popup, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(popup_area);

    f.render_widget(label_paragraph, inner[0]);
    f.render_widget(input_paragraph, inner[1]);

    f.set_cursor(
        inner[1].x + app.input.len() as u16 + 1,
        inner[1].y + 1,
    );
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let spans: Vec<Span> = match app.mode {
        Mode::CategoryList => keybinds(&[
            ("↑↓", "nav"), ("a", "add"), ("Esc", "back"), ("q", "quit"),
        ]),
        Mode::EventDetail => keybinds(&[
            ("Esc", "back"), ("d", "done"), ("q", "quit"),
        ]),
        Mode::CalendarView => keybinds(&[
            ("←→↑↓", "move"), ("t", "today"),
            ("jk", "nav"), ("Enter", "detail"),
            ("a", "add"), ("d", "done"), ("x", "del"), ("q", "quit"),
        ]),
        Mode::AddEvent | Mode::AddCategory => keybinds(&[
            ("Enter", "save"), ("Esc", "cancel"),
        ]),
    };

    let mut content = Line::from(spans);

    // Show message if any
    if !app.message.is_empty() {
        content = Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(&app.message, Style::default().fg(Color::Green)),
        ]);
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let p = Paragraph::new(content).block(block);
    f.render_widget(p, area);
}

fn keybinds<'a>(pairs: &[(&'a str, &'a str)]) -> Vec<Span<'a>> {
    let mut spans = vec![Span::raw(" ")];
    for (key, desc) in pairs {
        spans.push(Span::styled(*key, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}  ", desc), Style::default().fg(Color::DarkGray)));
    }
    spans
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

const DIM_ACCENT: Color = Color::Rgb(100, 20, 20);

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}