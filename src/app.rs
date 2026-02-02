use crate::config::Config;
use crate::heos::{
    AvrEvent, AvrHandle, BrowseItem, HeosEvent, HeosHandle, MusicSource, MuteState,
    NowPlayingMedia, PlayState, Player, PlayerState, QueueItem, RepeatMode, ShuffleMode,
    SurroundMode,
};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Main,
    Devices,
    Queue,
    Browse,
    Inputs,
    SurroundModes,
    SoundSettings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Discovering,
    Connected,
}

/// AVR-specific state
#[derive(Debug, Clone, Default)]
pub struct AvrState {
    pub connected: bool,
    pub power: bool,
    pub master_volume: u8,
    pub muted: bool,
    pub surround_mode: String,
    pub input_source: String,
}

pub struct App {
    pub config: Config,
    pub connection_state: ConnectionState,
    pub current_view: View,
    pub previous_view: View,
    pub should_quit: bool,
    pub status_message: Option<String>,

    // Player state (HEOS)
    pub players: Vec<Player>,
    pub current_player_idx: usize,
    pub player_state: PlayerState,

    // Queue
    pub queue: Vec<QueueItem>,
    pub queue_selected: usize,

    // Browse
    pub music_sources: Vec<MusicSource>,
    pub browse_items: Vec<BrowseItem>,
    pub browse_selected: usize,
    pub browse_stack: Vec<(i64, String)>, // (sid, cid) history

    // Inputs
    pub inputs: Vec<MusicSource>,
    pub input_selected: usize,

    // Device selection
    pub device_selected: usize,

    // Surround mode selection
    pub surround_selected: usize,

    // Sound settings selection
    pub sound_setting_selected: usize,

    // HEOS client handle
    handle: Option<HeosHandle>,

    // AVR control handle and state
    avr_handle: Option<AvrHandle>,
    pub avr_state: AvrState,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            connection_state: ConnectionState::Disconnected,
            current_view: View::Main,
            previous_view: View::Main,
            should_quit: false,
            status_message: None,
            players: Vec::new(),
            current_player_idx: 0,
            player_state: PlayerState::default(),
            queue: Vec::new(),
            queue_selected: 0,
            music_sources: Vec::new(),
            browse_items: Vec::new(),
            browse_selected: 0,
            browse_stack: Vec::new(),
            inputs: Vec::new(),
            input_selected: 0,
            device_selected: 0,
            surround_selected: 0,
            sound_setting_selected: 0,
            handle: None,
            avr_handle: None,
            avr_state: AvrState::default(),
        }
    }

    pub fn set_handle(&mut self, handle: HeosHandle) {
        self.handle = Some(handle);
        self.connection_state = ConnectionState::Connected;
    }

    pub fn get_handle(&self) -> Option<&HeosHandle> {
        self.handle.as_ref()
    }

    pub fn set_avr_handle(&mut self, handle: AvrHandle) {
        self.avr_handle = Some(handle);
        self.avr_state.connected = true;
    }

    pub fn current_player(&self) -> Option<&Player> {
        self.players.get(self.current_player_idx)
    }

    pub fn current_pid(&self) -> Option<i64> {
        self.current_player().map(|p| p.pid)
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn show_view(&mut self, view: View) {
        if self.current_view != view {
            self.previous_view = self.current_view;
            self.current_view = view;
        }
    }

    pub fn go_back(&mut self) {
        match self.current_view {
            View::Help | View::Devices | View::Queue | View::Inputs
            | View::SurroundModes | View::SoundSettings => {
                self.current_view = View::Main;
            }
            View::Browse => {
                if self.browse_stack.is_empty() {
                    self.current_view = View::Main;
                } else {
                    self.browse_stack.pop();
                }
            }
            View::Main => {}
        }
    }

    // ==================== HEOS Commands ====================

    pub async fn refresh_players(&self) -> Result<()> {
        if let Some(handle) = &self.handle {
            handle.get_players().await?;
        }
        Ok(())
    }

    pub async fn refresh_player_state(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.get_play_state(pid).await?;
            handle.get_now_playing(pid).await?;
            handle.get_volume(pid).await?;
            handle.get_mute(pid).await?;
            handle.get_play_mode(pid).await?;
        }
        Ok(())
    }

    pub async fn toggle_play_pause(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            match self.player_state.play_state {
                PlayState::Play => handle.pause(pid).await?,
                _ => handle.play(pid).await?,
            }
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.stop(pid).await?;
        }
        Ok(())
    }

    pub async fn next_track(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.play_next(pid).await?;
        }
        Ok(())
    }

    pub async fn prev_track(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.play_previous(pid).await?;
        }
        Ok(())
    }

    pub async fn volume_up(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.volume_up(pid, self.config.ui.volume_step).await?;
        }
        Ok(())
    }

    pub async fn volume_down(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.volume_down(pid, self.config.ui.volume_step).await?;
        }
        Ok(())
    }

    pub async fn toggle_mute(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.toggle_mute(pid).await?;
        }
        Ok(())
    }

    pub async fn cycle_repeat(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            let new_repeat = self.player_state.repeat.next();
            handle
                .set_play_mode(pid, new_repeat.as_str(), self.player_state.shuffle.as_str())
                .await?;
        }
        Ok(())
    }

    pub async fn toggle_shuffle(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            let new_shuffle = self.player_state.shuffle.toggle();
            handle
                .set_play_mode(pid, self.player_state.repeat.as_str(), new_shuffle.as_str())
                .await?;
        }
        Ok(())
    }

    pub async fn refresh_queue(&self) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.get_queue(pid, 0, 100).await?;
        }
        Ok(())
    }

    pub async fn play_queue_item(&self, qid: i64) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.play_queue_item(pid, qid).await?;
        }
        Ok(())
    }

    pub async fn refresh_music_sources(&self) -> Result<()> {
        if let Some(handle) = &self.handle {
            handle.get_music_sources().await?;
        }
        Ok(())
    }

    pub async fn browse_source(&self, sid: i64) -> Result<()> {
        if let Some(handle) = &self.handle {
            handle.browse_source(sid).await?;
        }
        Ok(())
    }

    pub async fn browse_container(&self, sid: i64, cid: &str) -> Result<()> {
        if let Some(handle) = &self.handle {
            handle.browse_container(sid, cid).await?;
        }
        Ok(())
    }

    pub async fn select_player(&mut self, idx: usize) -> Result<()> {
        if idx < self.players.len() {
            self.current_player_idx = idx;
            self.player_state = PlayerState::default();
            if let Some(player) = self.players.get(idx) {
                self.player_state.player = Some(player.clone());
            }
            self.refresh_player_state().await?;
        }
        Ok(())
    }

    pub async fn play_input(&self, input: &str) -> Result<()> {
        if let (Some(handle), Some(pid)) = (&self.handle, self.current_pid()) {
            handle.play_input(pid, input).await?;
        }
        Ok(())
    }

    // ==================== AVR Commands ====================

    pub async fn avr_query_status(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.query_status().await?;
        }
        Ok(())
    }

    pub async fn avr_set_surround_mode(&self, mode: SurroundMode) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.set_surround_mode(mode).await?;
        }
        Ok(())
    }

    pub async fn avr_set_input(&self, input: &str) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.set_input(input).await?;
        }
        Ok(())
    }

    pub async fn avr_volume_up(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.volume_up().await?;
        }
        Ok(())
    }

    pub async fn avr_volume_down(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.volume_down().await?;
        }
        Ok(())
    }

    pub async fn avr_mute_toggle(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            if self.avr_state.muted {
                avr.mute_off().await?;
            } else {
                avr.mute_on().await?;
            }
        }
        Ok(())
    }

    pub async fn avr_bass_up(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.bass_up().await?;
        }
        Ok(())
    }

    pub async fn avr_bass_down(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.bass_down().await?;
        }
        Ok(())
    }

    pub async fn avr_treble_up(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.treble_up().await?;
        }
        Ok(())
    }

    pub async fn avr_treble_down(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.treble_down().await?;
        }
        Ok(())
    }

    pub async fn avr_dynamic_eq_toggle(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            // Toggle - we'd need to track state properly
            avr.dynamic_eq_on().await?;
        }
        Ok(())
    }

    pub async fn avr_subwoofer_up(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.subwoofer_up().await?;
        }
        Ok(())
    }

    pub async fn avr_subwoofer_down(&self) -> Result<()> {
        if let Some(avr) = &self.avr_handle {
            avr.subwoofer_down().await?;
        }
        Ok(())
    }

    // ==================== Event Handlers ====================

    pub fn handle_heos_event(&mut self, event: HeosEvent) {
        match event {
            HeosEvent::Connected => {
                self.connection_state = ConnectionState::Connected;
                self.set_status("Connected to HEOS device");
            }
            HeosEvent::Disconnected => {
                self.connection_state = ConnectionState::Disconnected;
                self.set_status("Disconnected from HEOS device");
                self.handle = None;
            }
            HeosEvent::PlayersChanged(players) => {
                if !players.is_empty() {
                    self.players = players;
                }
            }
            HeosEvent::PlayerStateChanged { pid, state } => {
                if self.current_pid() == Some(pid) {
                    self.player_state.play_state = state;
                }
            }
            HeosEvent::NowPlayingChanged { pid } => {
                if self.current_pid() == Some(pid) {
                    // Trigger a refresh of now playing - handled by caller
                }
            }
            HeosEvent::VolumeChanged { pid, level, mute } => {
                if self.current_pid() == Some(pid) {
                    self.player_state.volume = level;
                    self.player_state.mute = mute;
                }
            }
            HeosEvent::PlayModeChanged { pid, repeat, shuffle } => {
                if self.current_pid() == Some(pid) {
                    self.player_state.repeat = repeat;
                    self.player_state.shuffle = shuffle;
                }
            }
            HeosEvent::QueueChanged { pid: _ } => {
                // Trigger queue refresh if viewing queue
            }
            HeosEvent::Error(msg) => {
                self.set_status(format!("Error: {}", msg));
            }
            HeosEvent::Response(response) => {
                self.handle_response(response);
            }
        }
    }

    pub fn handle_avr_event(&mut self, event: AvrEvent) {
        match event {
            AvrEvent::Connected => {
                self.avr_state.connected = true;
                self.set_status("AVR control connected");
            }
            AvrEvent::Disconnected => {
                self.avr_state.connected = false;
                self.avr_handle = None;
            }
            AvrEvent::MasterVolume(vol) => {
                self.avr_state.master_volume = vol;
            }
            AvrEvent::Mute(muted) => {
                self.avr_state.muted = muted;
            }
            AvrEvent::Power(on) => {
                self.avr_state.power = on;
            }
            AvrEvent::SurroundMode(mode) => {
                self.avr_state.surround_mode = mode;
            }
            AvrEvent::InputSource(input) => {
                self.avr_state.input_source = input;
            }
            AvrEvent::Error(msg) => {
                self.set_status(format!("AVR Error: {}", msg));
            }
            AvrEvent::Response(_) => {
                // Generic response, ignore
            }
        }
    }

    fn handle_response(&mut self, response: crate::heos::protocol::HeosResponse) {
        if !response.is_success() {
            let params = response.parse_message();
            if let Some(text) = params.get("text") {
                self.set_status(format!("Error: {}", text));
            }
            return;
        }

        let cmd = &response.heos.command;

        if cmd.contains("get_players") {
            if let Some(players) = response.get_payload_array::<Player>() {
                self.players = players;
                if !self.players.is_empty() && self.player_state.player.is_none() {
                    self.player_state.player = Some(self.players[0].clone());
                }
            }
        } else if cmd.contains("get_play_state") {
            let params = response.parse_message();
            if let Some(state) = params.get("state") {
                self.player_state.play_state = PlayState::from_str(state);
            }
        } else if cmd.contains("get_now_playing_media") {
            if let Some(media) = response.get_payload_object::<NowPlayingMedia>() {
                self.player_state.now_playing = media;
            }
        } else if cmd.contains("get_volume") || cmd.contains("volume_up") || cmd.contains("volume_down") {
            let params = response.parse_message();
            if let Some(level) = params.get("level").and_then(|s| s.parse().ok()) {
                self.player_state.volume = level;
            }
        } else if cmd.contains("get_mute") || cmd.contains("set_mute") || cmd.contains("toggle_mute") {
            let params = response.parse_message();
            if let Some(state) = params.get("state") {
                self.player_state.mute = MuteState::from_str(state);
            }
        } else if cmd.contains("get_play_mode") || cmd.contains("set_play_mode") {
            let params = response.parse_message();
            if let Some(repeat) = params.get("repeat") {
                self.player_state.repeat = RepeatMode::from_str(repeat);
            }
            if let Some(shuffle) = params.get("shuffle") {
                self.player_state.shuffle = ShuffleMode::from_str(shuffle);
            }
        } else if cmd.contains("get_queue") {
            if let Some(queue) = response.get_payload_array::<QueueItem>() {
                self.queue = queue;
            }
        } else if cmd.contains("get_music_sources") {
            if let Some(sources) = response.get_payload_array::<MusicSource>() {
                self.music_sources = sources
                    .iter()
                    .filter(|s| s.source_type != "heos_server")
                    .cloned()
                    .collect();
                self.inputs = sources
                    .into_iter()
                    .filter(|s| s.source_type == "heos_server" || s.name.contains("Input"))
                    .collect();
            }
        } else if cmd.contains("browse") {
            if let Some(items) = response.get_payload_array::<BrowseItem>() {
                self.browse_items = items;
                self.browse_selected = 0;
            }
        }
    }
}
