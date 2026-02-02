mod app;
mod config;
mod event;
mod heos;
mod ui;

use anyhow::{Context, Result};
use app::{App, ConnectionState, View};
use clap::Parser;
use config::Config;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{Action, AppEvent, EventHandler};
use heos::{discover_first_device, AvrClient, AvrEvent, AvrHandle, HeosClient, HeosEvent, HeosHandle};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about = "Terminal UI for HEOS devices")]
struct Args {
    /// HEOS device IP address (skips discovery)
    #[arg(short = 'H', long)]
    host: Option<String>,

    /// Discovery timeout in seconds
    #[arg(short, long, default_value = "5")]
    timeout: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load().unwrap_or_default();

    // Create event channels
    let (heos_tx, mut heos_rx) = mpsc::channel::<HeosEvent>(100);
    let (avr_tx, mut avr_rx) = mpsc::channel::<AvrEvent>(100);
    let (handle_tx, mut handle_rx) = mpsc::channel::<HeosHandle>(1);
    let (avr_handle_tx, mut avr_handle_rx) = mpsc::channel::<AvrHandle>(1);

    // Create app
    let mut app = App::new(config.clone());

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create event handler
    let tick_rate = Duration::from_millis(config.ui.refresh_rate);
    let mut event_handler = EventHandler::new(tick_rate);

    // Determine host to connect to
    let host = args.host.or(config.connection.host.clone());

    // Start connection/discovery
    app.connection_state = ConnectionState::Discovering;
    let connect_host = host.clone();
    let avr_host = host.clone();
    let connect_tx = heos_tx.clone();
    let discovery_timeout = args.timeout;

    // Spawn HEOS connection task
    tokio::spawn(async move {
        let target_host = if let Some(h) = connect_host {
            Some(h)
        } else {
            match discover_first_device(discovery_timeout).await {
                Ok(Some(ip)) => Some(ip),
                Ok(None) => None,
                Err(_) => None,
            }
        };

        if let Some(host) = target_host {
            match HeosClient::connect(&host, connect_tx.clone()).await {
                Ok(handle) => {
                    // Send handle back to main thread
                    let _ = handle_tx.send(handle.clone()).await;

                    // Register for events and get initial state
                    let _ = handle.register_for_events().await;
                    let _ = handle.get_players().await;
                }
                Err(e) => {
                    let _ = connect_tx
                        .send(HeosEvent::Error(format!("Connection failed: {}", e)))
                        .await;
                }
            }
        } else {
            let _ = connect_tx
                .send(HeosEvent::Error("No HEOS device found".to_string()))
                .await;
        }
    });

    // Spawn AVR connection task (uses same host)
    let avr_connect_tx = avr_tx.clone();

    tokio::spawn(async move {
        // Wait a bit for HEOS to connect first, or use provided host
        tokio::time::sleep(Duration::from_millis(500)).await;

        let target_host = if let Some(h) = avr_host {
            Some(h)
        } else {
            // Try discovery again for AVR
            match discover_first_device(3).await {
                Ok(Some(ip)) => Some(ip),
                _ => None,
            }
        };

        if let Some(host) = target_host {
            match AvrClient::connect(&host, avr_connect_tx.clone()).await {
                Ok(handle) => {
                    // Send handle back to main thread
                    let _ = avr_handle_tx.send(handle.clone()).await;

                    // Query initial status
                    let _ = handle.query_status().await;
                }
                Err(e) => {
                    let _ = avr_connect_tx
                        .send(AvrEvent::Error(format!("AVR connection failed: {}", e)))
                        .await;
                }
            }
        }
    });

    // Main event loop
    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Handle events
        tokio::select! {
            Some(app_event) = event_handler.next() => {
                match app_event {
                    AppEvent::Key(key) => {
                        if let Some(action) = Action::from_key(key) {
                            handle_action(&mut app, action).await?;
                        }
                    }
                    AppEvent::Tick => {
                        // Could clear old status messages here
                    }
                    AppEvent::Resize(_, _) => {
                        // Terminal will redraw on next iteration
                    }
                }
            }
            Some(heos_event) = heos_rx.recv() => {
                // Check if this is a now_playing_changed event and refresh
                let should_refresh_now_playing = matches!(
                    &heos_event,
                    HeosEvent::NowPlayingChanged { pid } if app.current_pid() == Some(*pid)
                );

                app.handle_heos_event(heos_event);

                // Auto-refresh now playing when it changes
                if should_refresh_now_playing {
                    if let Some(pid) = app.current_pid() {
                        if let Some(handle) = app.get_handle() {
                            let _ = handle.get_now_playing(pid).await;
                        }
                    }
                }
            }
            Some(avr_event) = avr_rx.recv() => {
                app.handle_avr_event(avr_event);
            }
            Some(handle) = handle_rx.recv() => {
                app.set_handle(handle.clone());
                // Get initial player state
                if let Err(e) = app.refresh_player_state().await {
                    app.set_status(format!("Error getting player state: {}", e));
                }
            }
            Some(avr_handle) = avr_handle_rx.recv() => {
                app.set_avr_handle(avr_handle);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    Ok(())
}

async fn handle_action(app: &mut App, action: Action) -> Result<()> {
    match action {
        Action::Quit => {
            app.should_quit = true;
        }
        Action::PlayPause => {
            if let Err(e) = app.toggle_play_pause().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::Stop => {
            if let Err(e) = app.stop().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::NextTrack => {
            if let Err(e) = app.next_track().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::PrevTrack => {
            if let Err(e) = app.prev_track().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::VolumeUp => {
            if let Err(e) = app.volume_up().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::VolumeDown => {
            if let Err(e) = app.volume_down().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ToggleMute => {
            if let Err(e) = app.toggle_mute().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::CycleRepeat => {
            if let Err(e) = app.cycle_repeat().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ToggleShuffle => {
            if let Err(e) = app.toggle_shuffle().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ShowDevices => {
            app.show_view(View::Devices);
            if let Err(e) = app.refresh_players().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ShowQueue => {
            app.show_view(View::Queue);
            if let Err(e) = app.refresh_queue().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ShowBrowse => {
            app.show_view(View::Browse);
            app.browse_stack.clear();
            if let Err(e) = app.refresh_music_sources().await {
                app.set_status(format!("Error: {}", e));
            }
        }
        Action::ShowInputs => {
            app.show_view(View::Inputs);
        }
        Action::ShowSurroundModes => {
            app.show_view(View::SurroundModes);
            app.surround_selected = 0;
        }
        Action::ShowSoundSettings => {
            app.show_view(View::SoundSettings);
            app.sound_setting_selected = 0;
        }
        Action::ShowHelp => {
            app.show_view(View::Help);
        }
        Action::Back => {
            app.go_back();
        }
        Action::Select => {
            handle_select(app).await?;
        }
        Action::MoveUp => {
            handle_move_up(app);
        }
        Action::MoveDown => {
            handle_move_down(app);
        }
        Action::MoveLeft | Action::MoveRight => {
            // Could be used for seeking in future
        }
        Action::Refresh => {
            if let Err(e) = app.refresh_player_state().await {
                app.set_status(format!("Error: {}", e));
            }
            if let Err(e) = app.avr_query_status().await {
                app.set_status(format!("Error: {}", e));
            }
        }
    }
    Ok(())
}

fn handle_move_up(app: &mut App) {
    match app.current_view {
        View::Devices => {
            if app.device_selected > 0 {
                app.device_selected -= 1;
            }
        }
        View::Queue => {
            if app.queue_selected > 0 {
                app.queue_selected -= 1;
            }
        }
        View::Browse => {
            if app.browse_selected > 0 {
                app.browse_selected -= 1;
            }
        }
        View::Inputs => {
            if app.input_selected > 0 {
                app.input_selected -= 1;
            }
        }
        View::SurroundModes => {
            if app.surround_selected > 0 {
                app.surround_selected -= 1;
            }
        }
        View::SoundSettings => {
            if app.sound_setting_selected > 0 {
                app.sound_setting_selected -= 1;
            }
        }
        _ => {}
    }
}

fn handle_move_down(app: &mut App) {
    match app.current_view {
        View::Devices => {
            if app.device_selected < app.players.len().saturating_sub(1) {
                app.device_selected += 1;
            }
        }
        View::Queue => {
            if app.queue_selected < app.queue.len().saturating_sub(1) {
                app.queue_selected += 1;
            }
        }
        View::Browse => {
            let max = if app.browse_stack.is_empty() {
                app.music_sources.len()
            } else {
                app.browse_items.len()
            };
            if app.browse_selected < max.saturating_sub(1) {
                app.browse_selected += 1;
            }
        }
        View::Inputs => {
            if app.input_selected < ui::inputs::input_count().saturating_sub(1) {
                app.input_selected += 1;
            }
        }
        View::SurroundModes => {
            if app.surround_selected < ui::surround::mode_count().saturating_sub(1) {
                app.surround_selected += 1;
            }
        }
        View::SoundSettings => {
            if app.sound_setting_selected < ui::sound_settings::setting_count().saturating_sub(1) {
                app.sound_setting_selected += 1;
            }
        }
        _ => {}
    }
}

async fn handle_select(app: &mut App) -> Result<()> {
    match app.current_view {
        View::Devices => {
            let idx = app.device_selected;
            if let Err(e) = app.select_player(idx).await {
                app.set_status(format!("Error: {}", e));
            }
            app.current_view = View::Main;
        }
        View::Queue => {
            if let Some(item) = app.queue.get(app.queue_selected) {
                let qid = item.qid;
                if let Err(e) = app.play_queue_item(qid).await {
                    app.set_status(format!("Error: {}", e));
                }
            }
        }
        View::Browse => {
            if app.browse_stack.is_empty() {
                // Select a music source
                if let Some(source) = app.music_sources.get(app.browse_selected) {
                    let sid = source.sid;
                    app.browse_stack.push((sid, source.name.clone()));
                    if let Err(e) = app.browse_source(sid).await {
                        app.set_status(format!("Error: {}", e));
                        app.browse_stack.pop();
                    }
                }
            } else {
                // Select a browse item
                if let Some(item) = app.browse_items.get(app.browse_selected) {
                    if item.container == "yes" {
                        if let Some((sid, _)) = app.browse_stack.last() {
                            let sid = *sid;
                            let cid = item.cid.clone();
                            app.browse_stack.push((sid, item.name.clone()));
                            if let Err(e) = app.browse_container(sid, &cid).await {
                                app.set_status(format!("Error: {}", e));
                                app.browse_stack.pop();
                            }
                        }
                    }
                    // TODO: Handle playable items
                }
            }
            app.browse_selected = 0;
        }
        View::Inputs => {
            if let Some(input) = ui::inputs::get_input_at_index(app.input_selected) {
                if let Err(e) = app.play_input(input).await {
                    app.set_status(format!("Error: {}", e));
                }
            }
            app.current_view = View::Main;
        }
        View::SurroundModes => {
            if let Some(mode) = ui::surround::get_mode_at_index(app.surround_selected) {
                if let Err(e) = app.avr_set_surround_mode(mode).await {
                    app.set_status(format!("Error: {}", e));
                } else {
                    app.set_status(format!("Surround mode: {}", mode.display_name()));
                }
            }
            app.current_view = View::Main;
        }
        View::SoundSettings => {
            if let Some(setting) = ui::sound_settings::get_setting_at_index(app.sound_setting_selected) {
                use ui::sound_settings::SoundSetting;
                let result = match setting {
                    SoundSetting::BassUp => app.avr_bass_up().await,
                    SoundSetting::BassDown => app.avr_bass_down().await,
                    SoundSetting::TrebleUp => app.avr_treble_up().await,
                    SoundSetting::TrebleDown => app.avr_treble_down().await,
                    SoundSetting::SubwooferUp => app.avr_subwoofer_up().await,
                    SoundSetting::SubwooferDown => app.avr_subwoofer_down().await,
                    SoundSetting::DynamicEq => app.avr_dynamic_eq_toggle().await,
                    SoundSetting::DialogEnhancer => {
                        // TODO: Could prompt for level
                        app.set_status("Dialog enhancer adjusted");
                        Ok(())
                    }
                };
                if let Err(e) = result {
                    app.set_status(format!("Error: {}", e));
                } else {
                    app.set_status(format!("Applied: {}", setting.display_name()));
                }
            }
            // Don't close - allow multiple adjustments
        }
        View::Help => {
            app.current_view = View::Main;
        }
        View::Main => {}
    }
    Ok(())
}
