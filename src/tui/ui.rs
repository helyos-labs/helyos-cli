use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::app::{ActivePanel, App, InputMode};
use super::widgets::{event_list, node_gauge, pod_table};

pub fn draw(f: &mut Frame, app: &App) {
    if let InputMode::LogView(name, lines) = &app.input_mode {
        draw_log_view(f, f.area(), name, lines);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Percentage(25),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_status_bar(f, chunks[0], app);
    draw_main_panels(f, chunks[1], app);
    event_list::render(f, chunks[2], app, app.active_panel == ActivePanel::Events);
    draw_keybinds(f, chunks[3], app);

    if app.show_help {
        draw_help_overlay(f, f.area());
    }
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_color = if app.connected {
        Color::Rgb(63, 185, 80)
    } else {
        Color::Rgb(248, 81, 73)
    };
    let status_text = if app.connected {
        "⏻ running"
    } else {
        "● disconnected"
    };

    let pod_count = app.pods.len();
    let deploy_count = app.deployments.len();
    let node_count = app.nodes.len();

    let line = Line::from(vec![
        Span::styled(
            " ⊞ Helyos ",
            Style::default()
                .fg(Color::Rgb(88, 166, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(status_text, Style::default().fg(status_color)),
        Span::styled(
            format!(
                "  │ ◆ {node_count} nodes │ ● {pod_count} pods │ ◎ {deploy_count} deploys │ ↻ 2s"
            ),
            Style::default().fg(Color::Rgb(139, 148, 158)),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(48, 54, 61)))
        .style(Style::default().bg(Color::Rgb(22, 27, 34)));

    let para = Paragraph::new(line).block(block);
    f.render_widget(para, area);
}

fn draw_main_panels(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let pod_table = pod_table::build_table(app, app.active_panel == ActivePanel::Pods);
    f.render_widget(pod_table, chunks[0]);
    node_gauge::render(f, chunks[1], app, app.active_panel == ActivePanel::Nodes);
}

fn draw_keybinds(f: &mut Frame, area: Rect, app: &App) {
    if let Some(msg) = &app.status_message {
        let style = if msg.starts_with('✓') {
            Style::default().fg(Color::Rgb(63, 185, 80))
        } else if msg.starts_with('✗') {
            Style::default().fg(Color::Rgb(248, 81, 73))
        } else {
            Style::default().fg(Color::Rgb(210, 153, 34))
        };
        let para = Paragraph::new(format!(" {msg}")).style(style);
        f.render_widget(para, area);
        return;
    }

    let line = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" quit  ", Style::default().fg(Color::Rgb(72, 79, 88))),
        Span::styled("?", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" help  ", Style::default().fg(Color::Rgb(72, 79, 88))),
        Span::styled("Tab", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" panel  ", Style::default().fg(Color::Rgb(72, 79, 88))),
        Span::styled("↑↓", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" nav  ", Style::default().fg(Color::Rgb(72, 79, 88))),
        Span::styled("d", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" delete  ", Style::default().fg(Color::Rgb(72, 79, 88))),
        Span::styled("s", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" scale", Style::default().fg(Color::Rgb(72, 79, 88))),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let popup_width = 50u16.min(area.width - 4);
    let popup_height = 14u16.min(area.height - 4);
    let x = (area.width - popup_width) / 2;
    let y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab       ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Cycle panels"),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑↓ / jk   ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Navigate"),
        ]),
        Line::from(vec![
            Span::styled(
                "  d         ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Delete selected pod"),
        ]),
        Line::from(vec![
            Span::styled(
                "  s         ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Scale deployment"),
        ]),
        Line::from(vec![
            Span::styled(
                "  ?         ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled(
                "  q / Esc   ",
                Style::default().fg(Color::Rgb(88, 166, 255)),
            ),
            Span::raw("Quit"),
        ]),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(88, 166, 255)))
        .title(" ? Help ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(88, 166, 255))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));

    let para = Paragraph::new(help_text).block(block);
    f.render_widget(para, popup_area);
}

fn draw_log_view(f: &mut Frame, area: Rect, name: &str, lines: &[String]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(88, 166, 255)))
        .title(format!(" ● Logs — {name} "))
        .title_style(
            Style::default()
                .fg(Color::Rgb(88, 166, 255))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));

    let visible_height = (chunks[0].height as usize).saturating_sub(2);
    let start = lines.len().saturating_sub(visible_height);
    let visible_lines: Vec<Line> = lines[start..]
        .iter()
        .map(|line| {
            Line::from(vec![
                Span::styled("│ ", Style::default().fg(Color::Rgb(48, 54, 61))),
                Span::styled(
                    line.as_str(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
            ])
        })
        .collect();

    let para = Paragraph::new(visible_lines).block(block);
    f.render_widget(para, chunks[0]);

    let keybinds = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Rgb(201, 209, 217))),
        Span::styled(" back", Style::default().fg(Color::Rgb(72, 79, 88))),
    ]);
    let footer = Paragraph::new(keybinds);
    f.render_widget(footer, chunks[1]);
}
