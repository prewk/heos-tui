use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Queue list
        Constraint::Length(1), // Instructions
    ])
    .split(frame.area());

    // Header
    let header = Paragraph::new(format!(" Queue ({} items)", app.queue.len()))
        .style(Style::default().bold())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    frame.render_widget(header, chunks[0]);

    // Queue list
    let items: Vec<ListItem> = app
        .queue
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_highlighted = i == app.queue_selected;
            let is_current = app.player_state.now_playing.qid == item.qid;

            let prefix = if is_current { "▶ " } else { "  " };
            let content = format!(
                "{}{:3}. {} - {}",
                prefix,
                i + 1,
                item.song,
                item.artist
            );

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if is_current {
                Style::default().fg(Color::Cyan)
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
                .title(" Queue ")
                .title_alignment(Alignment::Left),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, chunks[1]);

    // Instructions
    let instructions = " ↑/↓ Navigate  Enter Play  Esc Back  c Clear queue ";
    let instructions_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(instructions_para, chunks[2]);
}
