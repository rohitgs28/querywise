use ratatui::{prelude::*, widgets::*};
use crate::app::{App, Panel};

pub fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_title_bar(frame, outer[0]);
    render_main(frame, app, outer[1]);
    render_status_bar(frame, app, outer[2]);
    render_key_hints(frame, app, outer[3]);
}

fn render_title_bar(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            " QueryWise ",
            Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
        ),
        Span::styled(
            " AI-Powered Database Client ",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(title).bg(Color::DarkGray), area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    render_schema_panel(frame, app, main_layout[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(main_layout[1]);

    render_chat_panel(frame, app, right[0]);
    render_sql_panel(frame, app, right[1]);
    render_results_panel(frame, app, right[2]);
}

fn render_schema_panel(frame: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::Schema;
    let border = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .schema
        .tables
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            let mut items = vec![];
            let ts = if i == app.selected_table {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(&t.name, ts),
                Span::styled(
                    format!(" ({})", t.row_count),
                    Style::default().fg(Color::DarkGray),
                ),
            ])));
            if i == app.selected_table {
                for col in &t.columns {
                    let tc = if col.is_primary_key {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    };
                    let pk = if col.is_primary_key { " PK" } else { "" };
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(&col.name, Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!(" {}{}", col.data_type, pk),
                            Style::default().fg(tc),
                        ),
                    ])));
                }
            }
            items
        })
        .collect();

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(" Schema [F1] ")
                .borders(Borders::ALL)
                .border_style(border),
        ),
        area,
    );
}

fn render_chat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::AiChat;
    let border = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let messages: Vec<ListItem> = app
        .chat_messages
        .iter()
        .map(|m| {
            let (prefix, style) = match m.role.as_str() {
                "user" => ("> ", Style::default().fg(Color::White).bold()),
                "sql" => ("  SQL: ", Style::default().fg(Color::Green)),
                "result" => ("  ✓ ", Style::default().fg(Color::Cyan)),
                "error" => ("  ERR: ", Style::default().fg(Color::Red)),
                "info" => ("  ℹ ", Style::default().fg(Color::Yellow)),
                _ => ("  ", Style::default()),
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, m.content),
                style,
            )))
        })
        .collect();

    frame.render_widget(
        List::new(messages).block(
            Block::default()
                .title(" AI Chat [F2] ")
                .borders(Borders::ALL)
                .border_style(border),
        ),
        inner[0],
    );

    // Input title changes when navigating history
    let input_title = if app.query_history.is_navigating() {
        " History (↑↓ navigate, Enter to run) "
    } else {
        " Ask a question, enter SQL, or :explain "
    };
    let input_border = if active {
        if app.query_history.is_navigating() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        }
    } else {
        Style::default().fg(Color::DarkGray)
    };

    frame.render_widget(
        Paragraph::new(app.query_input.as_str())
            .block(
                Block::default()
                    .title(input_title)
                    .borders(Borders::ALL)
                    .border_style(input_border),
            )
            .style(Style::default().fg(Color::White)),
        inner[1],
    );

    if active {
        frame.set_cursor_position((inner[1].x + app.cursor_pos as u16 + 1, inner[1].y + 1));
    }
}

fn render_sql_panel(frame: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::SqlPreview;
    let border = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let sql = if app.generated_sql.is_empty() {
        "No query generated yet".to_string()
    } else {
        app.generated_sql.clone()
    };

    frame.render_widget(
        Paragraph::new(sql)
            .block(
                Block::default()
                    .title(" SQL [F3] ")
                    .borders(Borders::ALL)
                    .border_style(border),
            )
            .style(Style::default().fg(Color::Green))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_results_panel(frame: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::Results;
    let border = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if app.result_columns.is_empty() {
        frame.render_widget(
            Paragraph::new("Execute a query to see results")
                .block(
                    Block::default()
                        .title(" Results [F4] ")
                        .borders(Borders::ALL)
                        .border_style(border),
                )
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let header = Row::new(
        app.result_columns
            .iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Cyan).bold())),
    )
    .height(1)
    .bottom_margin(1);

    let widths: Vec<Constraint> = app
        .result_columns
        .iter()
        .map(|_| Constraint::Min(12))
        .collect();

    let rows: Vec<Row> = app
        .result_rows
        .iter()
        .skip(app.result_scroll)
        .take(area.height.saturating_sub(4) as usize)
        .map(|row| {
            Row::new(
                row.iter()
                    .map(|c| {
                        let s = if c == "NULL" {
                            Style::default().fg(Color::DarkGray).italic()
                        } else {
                            Style::default().fg(Color::White)
                        };
                        Cell::from(c.as_str()).style(s)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let title = format!(
        " Results [F4] | {} ",
        if app.result_info.is_empty() {
            "No data"
        } else {
            &app.result_info
        }
    );

    frame.render_widget(
        Table::new(rows, &widths)
            .header(header)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border),
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let safe = if app.safe_mode {
        Span::styled(
            " SAFE ",
            Style::default().fg(Color::Black).bg(Color::Green),
        )
    } else {
        Span::styled(
            " UNSAFE ",
            Style::default().fg(Color::Black).bg(Color::Red),
        )
    };

    let hist = if app.query_history.len() > 0 {
        Span::styled(
            format!(" {} queries ", app.query_history.len()),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::styled("", Style::default())
    };

    let status = Line::from(vec![
        safe,
        Span::styled(
            format!(" {} ", app.status),
            Style::default().fg(Color::Gray),
        ),
        hist,
    ]);
    frame.render_widget(Paragraph::new(status).bg(Color::DarkGray), area);
}

fn render_key_hints(frame: &mut Frame, app: &App, area: Rect) {
    let hints = if app.active_panel == Panel::Results {
        Line::from(vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Switch ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ↑↓", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Scroll ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+E", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Export ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+S", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Safe ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+Q", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Quit ", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Switch ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Enter", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Run ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ↑↓", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" History ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+S", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Safe ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+L", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Clear ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+Q", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" Quit ", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(hints), area);
}
