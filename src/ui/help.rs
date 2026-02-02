use crate::app::App;
use crate::ui::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, _app: &App) {
    let area = centered_rect(70, 85, frame.area());

    // Clear the popup area
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Playback Controls",
            Style::default().bold().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Space / p  ", Style::default().fg(Color::Yellow)),
            Span::raw("Play / Pause"),
        ]),
        Line::from(vec![
            Span::styled("  s          ", Style::default().fg(Color::Yellow)),
            Span::raw("Stop"),
        ]),
        Line::from(vec![
            Span::styled("  n / Ctrl+→ ", Style::default().fg(Color::Yellow)),
            Span::raw("Next track"),
        ]),
        Line::from(vec![
            Span::styled("  b / Ctrl+← ", Style::default().fg(Color::Yellow)),
            Span::raw("Previous track"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Volume & Audio",
            Style::default().bold().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  + / =      ", Style::default().fg(Color::Yellow)),
            Span::raw("Volume up"),
        ]),
        Line::from(vec![
            Span::styled("  -          ", Style::default().fg(Color::Yellow)),
            Span::raw("Volume down"),
        ]),
        Line::from(vec![
            Span::styled("  m          ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle mute"),
        ]),
        Line::from(vec![
            Span::styled("  r          ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle repeat (off → all → one)"),
        ]),
        Line::from(vec![
            Span::styled("  z          ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle shuffle"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "AVR Controls",
            Style::default().bold().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  a          ", Style::default().fg(Color::Yellow)),
            Span::raw("Surround mode selector"),
        ]),
        Line::from(vec![
            Span::styled("  w          ", Style::default().fg(Color::Yellow)),
            Span::raw("Sound settings (bass, treble, etc.)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().bold().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  d          ", Style::default().fg(Color::Yellow)),
            Span::raw("Device selector"),
        ]),
        Line::from(vec![
            Span::styled("  u          ", Style::default().fg(Color::Yellow)),
            Span::raw("Queue view"),
        ]),
        Line::from(vec![
            Span::styled("  o          ", Style::default().fg(Color::Yellow)),
            Span::raw("Browse music sources"),
        ]),
        Line::from(vec![
            Span::styled("  i          ", Style::default().fg(Color::Yellow)),
            Span::raw("HEOS input selector"),
        ]),
        Line::from(vec![
            Span::styled("  ?          ", Style::default().fg(Color::Yellow)),
            Span::raw("Show this help"),
        ]),
        Line::from(vec![
            Span::styled("  Esc        ", Style::default().fg(Color::Yellow)),
            Span::raw("Go back / Close popup"),
        ]),
        Line::from(vec![
            Span::styled("  F5         ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh status"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+c ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "List Navigation",
            Style::default().bold().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑ / k      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓ / j      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter      ", Style::default().fg(Color::Yellow)),
            Span::raw("Select / Apply"),
        ]),
    ];

    let para = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Help ")
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(para, area);
}
