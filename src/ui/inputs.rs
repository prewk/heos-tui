use crate::app::App;
use crate::ui::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, frame.area());

    // Clear the popup area
    frame.render_widget(Clear, area);

    // Common inputs for Denon AVR
    let common_inputs = vec![
        ("HDMI 1", "inputs/hdmi_in_1"),
        ("HDMI 2", "inputs/hdmi_in_2"),
        ("HDMI 3", "inputs/hdmi_in_3"),
        ("HDMI 4", "inputs/hdmi_in_4"),
        ("HDMI 5", "inputs/hdmi_in_5"),
        ("HDMI 6", "inputs/hdmi_in_6"),
        ("TV Audio", "inputs/tv_audio"),
        ("Optical 1", "inputs/optical_in_1"),
        ("Optical 2", "inputs/optical_in_2"),
        ("Coax 1", "inputs/coaxial_in_1"),
        ("Aux 1", "inputs/aux_in_1"),
        ("Aux 2", "inputs/aux_in_2"),
        ("Bluetooth", "inputs/bluetooth"),
        ("Tuner", "inputs/tuner"),
        ("Phono", "inputs/phono"),
        ("CD", "inputs/cd"),
    ];

    let items: Vec<ListItem> = common_inputs
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            let is_highlighted = i == app.input_selected;

            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(format!("  {}  ", name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Select Input ")
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

pub fn get_input_at_index(index: usize) -> Option<&'static str> {
    let common_inputs = vec![
        "inputs/hdmi_in_1",
        "inputs/hdmi_in_2",
        "inputs/hdmi_in_3",
        "inputs/hdmi_in_4",
        "inputs/hdmi_in_5",
        "inputs/hdmi_in_6",
        "inputs/tv_audio",
        "inputs/optical_in_1",
        "inputs/optical_in_2",
        "inputs/coaxial_in_1",
        "inputs/aux_in_1",
        "inputs/aux_in_2",
        "inputs/bluetooth",
        "inputs/tuner",
        "inputs/phono",
        "inputs/cd",
    ];

    common_inputs.get(index).copied()
}

pub fn input_count() -> usize {
    16
}
