use crate::app::App;
use crate::ui::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.area());

    // Clear the popup area
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let is_selected = i == app.current_player_idx;
            let is_highlighted = i == app.device_selected;

            let prefix = if is_selected { "● " } else { "  " };
            let content = format!("{}{} ({})", prefix, player.name, player.model);

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if is_selected {
                Style::default().fg(Color::Green)
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
                .title(" Select Device ")
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);

    // Instructions
    let instructions = " ↑/↓ Navigate  Enter Select  Esc Cancel ";
    let instructions_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };

    let instructions_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(instructions_para, instructions_area);
}
