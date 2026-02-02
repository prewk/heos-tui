use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Browse list
        Constraint::Length(1), // Instructions
    ])
    .split(frame.area());

    // Header with breadcrumb
    let breadcrumb = if app.browse_stack.is_empty() {
        "Music Sources".to_string()
    } else {
        let path: Vec<String> = app
            .browse_stack
            .iter()
            .map(|(_, cid)| cid.clone())
            .collect();
        format!("Music Sources > {}", path.join(" > "))
    };

    let header = Paragraph::new(format!(" {}", breadcrumb))
        .style(Style::default().bold())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    frame.render_widget(header, chunks[0]);

    // Browse list - show sources if at root, otherwise show browse items
    if app.browse_stack.is_empty() {
        render_sources(frame, app, chunks[1]);
    } else {
        render_items(frame, app, chunks[1]);
    }

    // Instructions
    let instructions = " ↑/↓ Navigate  Enter Select/Play  Esc Back ";
    let instructions_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(instructions_para, chunks[2]);
}

fn render_sources(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .music_sources
        .iter()
        .enumerate()
        .map(|(i, source)| {
            let is_highlighted = i == app.browse_selected;

            let icon = match source.source_type.as_str() {
                "music_service" => "♪",
                "heos_server" => "📁",
                "dlna_server" => "💻",
                _ => "•",
            };

            let content = format!("{} {}", icon, source.name);

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Sources ")
                .title_alignment(Alignment::Left),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);
}

fn render_items(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .browse_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_highlighted = i == app.browse_selected;

            let icon = if item.container == "yes" {
                "📁"
            } else if item.playable == "yes" {
                "♪"
            } else {
                "•"
            };

            let content = format!("{} {}", icon, item.name);

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Browse ")
                .title_alignment(Alignment::Left),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);
}
