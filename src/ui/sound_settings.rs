use crate::app::App;
use crate::ui::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSetting {
    BassUp,
    BassDown,
    TrebleUp,
    TrebleDown,
    SubwooferUp,
    SubwooferDown,
    DynamicEq,
    DialogEnhancer,
}

impl SoundSetting {
    pub fn all() -> &'static [SoundSetting] {
        &[
            SoundSetting::BassUp,
            SoundSetting::BassDown,
            SoundSetting::TrebleUp,
            SoundSetting::TrebleDown,
            SoundSetting::SubwooferUp,
            SoundSetting::SubwooferDown,
            SoundSetting::DynamicEq,
            SoundSetting::DialogEnhancer,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SoundSetting::BassUp => "Bass +",
            SoundSetting::BassDown => "Bass -",
            SoundSetting::TrebleUp => "Treble +",
            SoundSetting::TrebleDown => "Treble -",
            SoundSetting::SubwooferUp => "Subwoofer +",
            SoundSetting::SubwooferDown => "Subwoofer -",
            SoundSetting::DynamicEq => "Dynamic EQ Toggle",
            SoundSetting::DialogEnhancer => "Dialog Enhancer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SoundSetting::BassUp => "Increase bass level",
            SoundSetting::BassDown => "Decrease bass level",
            SoundSetting::TrebleUp => "Increase treble level",
            SoundSetting::TrebleDown => "Decrease treble level",
            SoundSetting::SubwooferUp => "Increase subwoofer level",
            SoundSetting::SubwooferDown => "Decrease subwoofer level",
            SoundSetting::DynamicEq => "Toggle Audyssey Dynamic EQ",
            SoundSetting::DialogEnhancer => "Enhance dialog clarity",
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, frame.area());

    // Clear the popup area
    frame.render_widget(Clear, area);

    let settings = SoundSetting::all();

    let items: Vec<ListItem> = settings
        .iter()
        .enumerate()
        .map(|(i, setting)| {
            let is_highlighted = i == app.sound_setting_selected;

            let icon = match setting {
                SoundSetting::BassUp | SoundSetting::TrebleUp | SoundSetting::SubwooferUp => "▲",
                SoundSetting::BassDown | SoundSetting::TrebleDown | SoundSetting::SubwooferDown => "▼",
                SoundSetting::DynamicEq => "◐",
                SoundSetting::DialogEnhancer => "💬",
            };

            let content = format!("  {} {}  ", icon, setting.display_name());

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
                .title(" Sound Settings ")
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);

    // Show description for selected item
    if let Some(setting) = settings.get(app.sound_setting_selected) {
        let desc_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 3,
            width: area.width - 2,
            height: 1,
        };

        let desc = Paragraph::new(setting.description())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        frame.render_widget(desc, desc_area);
    }

    // Instructions
    let instructions = " ↑/↓ Navigate  Enter Apply  Esc Cancel ";
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

pub fn get_setting_at_index(index: usize) -> Option<SoundSetting> {
    SoundSetting::all().get(index).copied()
}

pub fn setting_count() -> usize {
    SoundSetting::all().len()
}
