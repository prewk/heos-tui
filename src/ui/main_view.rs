use crate::app::{App, ConnectionState};
use crate::heos::{MuteState, PlayState, RepeatMode, ShuffleMode};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title bar
        Constraint::Min(8),    // Now playing
        Constraint::Length(3), // Volume
        Constraint::Length(3), // AVR status (surround mode, input)
        Constraint::Length(3), // Controls
        Constraint::Length(1), // Status bar
    ])
    .split(frame.area());

    render_title_bar(frame, app, chunks[0]);
    render_now_playing(frame, app, chunks[1]);
    render_volume(frame, app, chunks[2]);
    render_avr_status(frame, app, chunks[3]);
    render_controls(frame, app, chunks[4]);
    render_status_bar(frame, app, chunks[5]);
}

fn render_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let player_name = app
        .current_player()
        .map(|p| p.name.as_str())
        .unwrap_or("No Player");

    let conn_status = match app.connection_state {
        ConnectionState::Connected => "●",
        ConnectionState::Discovering => "◐",
        ConnectionState::Disconnected => "○",
    };

    let conn_color = match app.connection_state {
        ConnectionState::Connected => Color::Green,
        ConnectionState::Discovering => Color::Yellow,
        ConnectionState::Disconnected => Color::Red,
    };

    // AVR connection indicator
    let avr_status = if app.avr_state.connected { "●" } else { "○" };
    let avr_color = if app.avr_state.connected {
        Color::Green
    } else {
        Color::DarkGray
    };

    let title = Line::from(vec![
        Span::styled(conn_status, Style::default().fg(conn_color)),
        Span::raw(" HEOS  "),
        Span::styled(avr_status, Style::default().fg(avr_color)),
        Span::raw(" AVR  │  "),
        Span::styled(player_name, Style::default().bold()),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" HEOS TUI ")
        .title_alignment(Alignment::Center);

    let para = Paragraph::new(title).block(block).alignment(Alignment::Center);

    frame.render_widget(para, area);
}

fn render_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let media = &app.player_state.now_playing;

    let play_icon = match app.player_state.play_state {
        PlayState::Play => "▶",
        PlayState::Pause => "⏸",
        PlayState::Stop => "⏹",
        PlayState::Unknown => "?",
    };

    let song = if media.song.is_empty() {
        "No media playing"
    } else {
        &media.song
    };

    let artist = if media.artist.is_empty() {
        "-"
    } else {
        &media.artist
    };

    let album = if media.album.is_empty() {
        "-"
    } else {
        &media.album
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(play_icon, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(song, Style::default().bold().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Artist: ", Style::default().fg(Color::DarkGray)),
            Span::raw(artist),
        ]),
        Line::from(vec![
            Span::styled("Album:  ", Style::default().fg(Color::DarkGray)),
            Span::raw(album),
        ]),
    ];

    // Add station info if available
    let mut display_lines = lines;
    if !media.station.is_empty() {
        display_lines.push(Line::from(vec![
            Span::styled("Station: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&media.station),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Now Playing ")
        .title_alignment(Alignment::Left);

    let para = Paragraph::new(display_lines).block(block);

    frame.render_widget(para, area);
}

fn render_volume(frame: &mut Frame, app: &App, area: Rect) {
    let volume = app.player_state.volume;
    let is_muted = app.player_state.mute == MuteState::On;

    let mute_indicator = if is_muted {
        Span::styled(" 🔇 MUTED ", Style::default().fg(Color::Red))
    } else {
        Span::styled(" 🔊 ", Style::default().fg(Color::Green))
    };

    let volume_text = format!("{}%", volume);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Volume "),
        )
        .gauge_style(
            Style::default()
                .fg(if is_muted { Color::DarkGray } else { Color::Cyan })
                .bg(Color::Black),
        )
        .percent(volume as u16)
        .label(Span::styled(
            volume_text,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let chunks = Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).split(area);

    let mute_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let mute_para = Paragraph::new(mute_indicator)
        .block(mute_block)
        .alignment(Alignment::Center);

    frame.render_widget(mute_para, chunks[0]);
    frame.render_widget(gauge, chunks[1]);
}

fn render_avr_status(frame: &mut Frame, app: &App, area: Rect) {
    let surround = if app.avr_state.surround_mode.is_empty() {
        "---".to_string()
    } else {
        app.avr_state.surround_mode.clone()
    };

    let input = if app.avr_state.input_source.is_empty() {
        "---".to_string()
    } else {
        app.avr_state.input_source.clone()
    };

    let avr_vol = format!("{}dB", app.avr_state.master_volume as i32 - 80);

    let content = Line::from(vec![
        Span::styled("[a]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Surround: "),
        Span::styled(&surround, Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled("[w]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Sound  │  Input: "),
        Span::styled(&input, Style::default().fg(Color::Yellow)),
        Span::raw("  │  AVR Vol: "),
        Span::styled(&avr_vol, Style::default().fg(Color::Green)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" AVR ");

    let para = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(para, area);
}

fn render_controls(frame: &mut Frame, app: &App, area: Rect) {
    let repeat_icon = match app.player_state.repeat {
        RepeatMode::Off => "↻",
        RepeatMode::OnAll => "🔁",
        RepeatMode::OnOne => "🔂",
    };

    let repeat_color = match app.player_state.repeat {
        RepeatMode::Off => Color::DarkGray,
        _ => Color::Green,
    };

    let shuffle_icon = if app.player_state.shuffle == ShuffleMode::On {
        "🔀"
    } else {
        "⇉"
    };

    let shuffle_color = if app.player_state.shuffle == ShuffleMode::On {
        Color::Green
    } else {
        Color::DarkGray
    };

    let controls = Line::from(vec![
        Span::styled("[b]", Style::default().fg(Color::DarkGray)),
        Span::raw(" ⏮ "),
        Span::styled("[p]", Style::default().fg(Color::DarkGray)),
        Span::raw(" ⏯ "),
        Span::styled("[n]", Style::default().fg(Color::DarkGray)),
        Span::raw(" ⏭  │  "),
        Span::styled("[r]", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(repeat_icon, Style::default().fg(repeat_color)),
        Span::raw("  "),
        Span::styled("[z]", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(shuffle_icon, Style::default().fg(shuffle_color)),
        Span::raw("  │  "),
        Span::styled("[d]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Devices  "),
        Span::styled("[u]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Queue  "),
        Span::styled("[?]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Help"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let para = Paragraph::new(controls)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(para, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = app
        .status_message
        .as_deref()
        .unwrap_or("Press ? for help");

    let para = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(para, area);
}
