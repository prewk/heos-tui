pub mod browse;
pub mod devices;
pub mod help;
pub mod inputs;
pub mod main_view;
pub mod queue;
pub mod sound_settings;
pub mod surround;

use crate::app::{App, View};
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &App) {
    match app.current_view {
        View::Main => main_view::render(frame, app),
        View::Devices => {
            main_view::render(frame, app);
            devices::render(frame, app);
        }
        View::Queue => queue::render(frame, app),
        View::Browse => browse::render(frame, app),
        View::Inputs => {
            main_view::render(frame, app);
            inputs::render(frame, app);
        }
        View::SurroundModes => {
            main_view::render(frame, app);
            surround::render(frame, app);
        }
        View::SoundSettings => {
            main_view::render(frame, app);
            sound_settings::render(frame, app);
        }
        View::Help => {
            main_view::render(frame, app);
            help::render(frame, app);
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
