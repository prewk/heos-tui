use crate::app::App;
use crate::heos::SurroundMode;
use crate::ui::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 70, frame.area());

    // Clear the popup area
    frame.render_widget(Clear, area);

    let modes = SurroundMode::all();

    let items: Vec<ListItem> = modes
        .iter()
        .enumerate()
        .map(|(i, mode)| {
            let is_highlighted = i == app.surround_selected;
            let is_current = app.avr_state.surround_mode.to_uppercase()
                == mode.display_name().to_uppercase()
                || app.avr_state.surround_mode.contains(&mode.display_name().to_uppercase());

            let prefix = if is_current { "● " } else { "  " };
            let content = format!("{}{}", prefix, mode.display_name());

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let current_mode = if app.avr_state.surround_mode.is_empty() {
        "Unknown".to_string()
    } else {
        app.avr_state.surround_mode.clone()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Surround Mode [{}] ", current_mode))
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

pub fn get_mode_at_index(index: usize) -> Option<SurroundMode> {
    SurroundMode::all().get(index).copied()
}

pub fn mode_count() -> usize {
    SurroundMode::all().len()
}
