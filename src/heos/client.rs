use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use super::protocol::{self, HeosCommand, HeosResponse};
use super::types::*;

pub const HEOS_PORT: u16 = 1255;

#[derive(Debug)]
pub enum HeosEvent {
    Connected,
    Disconnected,
    PlayersChanged(Vec<Player>),
    PlayerStateChanged { pid: i64, state: PlayState },
    NowPlayingChanged { pid: i64 },
    VolumeChanged { pid: i64, level: u8, mute: MuteState },
    PlayModeChanged { pid: i64, repeat: RepeatMode, shuffle: ShuffleMode },
    QueueChanged { pid: i64 },
    Error(String),
    Response(HeosResponse),
}

/// Handle for sending commands to the HEOS client
#[derive(Clone)]
pub struct HeosHandle {
    cmd_tx: mpsc::Sender<HeosCommand>,
}

impl HeosHandle {
    pub async fn send(&self, cmd: HeosCommand) -> Result<()> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| anyhow::anyhow!("Client disconnected"))
    }

    pub async fn register_for_events(&self) -> Result<()> {
        self.send(protocol::register_for_change_events(true)).await
    }

    pub async fn get_players(&self) -> Result<()> {
        self.send(protocol::get_players()).await
    }

    pub async fn get_play_state(&self, pid: i64) -> Result<()> {
        self.send(protocol::get_play_state(pid)).await
    }

    pub async fn set_play_state(&self, pid: i64, state: &str) -> Result<()> {
        self.send(protocol::set_play_state(pid, state)).await
    }

    pub async fn play(&self, pid: i64) -> Result<()> {
        self.set_play_state(pid, "play").await
    }

    pub async fn pause(&self, pid: i64) -> Result<()> {
        self.set_play_state(pid, "pause").await
    }

    pub async fn stop(&self, pid: i64) -> Result<()> {
        self.set_play_state(pid, "stop").await
    }

    pub async fn play_next(&self, pid: i64) -> Result<()> {
        self.send(protocol::play_next(pid)).await
    }

    pub async fn play_previous(&self, pid: i64) -> Result<()> {
        self.send(protocol::play_previous(pid)).await
    }

    pub async fn get_now_playing(&self, pid: i64) -> Result<()> {
        self.send(protocol::get_now_playing_media(pid)).await
    }

    pub async fn get_volume(&self, pid: i64) -> Result<()> {
        self.send(protocol::get_volume(pid)).await
    }

    pub async fn volume_up(&self, pid: i64, step: u8) -> Result<()> {
        self.send(protocol::volume_up(pid, step)).await
    }

    pub async fn volume_down(&self, pid: i64, step: u8) -> Result<()> {
        self.send(protocol::volume_down(pid, step)).await
    }

    pub async fn toggle_mute(&self, pid: i64) -> Result<()> {
        self.send(protocol::toggle_mute(pid)).await
    }

    pub async fn get_mute(&self, pid: i64) -> Result<()> {
        self.send(protocol::get_mute(pid)).await
    }

    pub async fn get_play_mode(&self, pid: i64) -> Result<()> {
        self.send(protocol::get_play_mode(pid)).await
    }

    pub async fn set_play_mode(&self, pid: i64, repeat: &str, shuffle: &str) -> Result<()> {
        self.send(protocol::set_play_mode(pid, repeat, shuffle))
            .await
    }

    pub async fn get_queue(&self, pid: i64, start: u32, end: u32) -> Result<()> {
        self.send(protocol::get_queue(pid, start, end)).await
    }

    pub async fn play_queue_item(&self, pid: i64, qid: i64) -> Result<()> {
        self.send(protocol::play_queue(pid, qid)).await
    }

    pub async fn get_music_sources(&self) -> Result<()> {
        self.send(protocol::get_music_sources()).await
    }

    pub async fn browse_source(&self, sid: i64) -> Result<()> {
        self.send(protocol::browse_source(sid)).await
    }

    pub async fn browse_container(&self, sid: i64, cid: &str) -> Result<()> {
        self.send(protocol::browse_source_container(sid, cid)).await
    }

    pub async fn play_input(&self, pid: i64, input: &str) -> Result<()> {
        self.send(protocol::play_input(pid, input)).await
    }
}

pub struct HeosClient {
    write_half: Arc<Mutex<Option<tokio::net::tcp::OwnedWriteHalf>>>,
}

impl HeosClient {
    pub async fn connect(
        host: &str,
        event_tx: mpsc::Sender<HeosEvent>,
    ) -> Result<HeosHandle> {
        let addr = format!("{}:{}", host, HEOS_PORT);
        let stream = TcpStream::connect(&addr)
            .await
            .context("Failed to connect to HEOS device")?;

        let (read_half, write_half) = stream.into_split();
        let write_half = Arc::new(Mutex::new(Some(write_half)));

        // Create command channel
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<HeosCommand>(100);

        // Spawn reader task
        let event_tx_clone = event_tx.clone();
        let write_half_clone = write_half.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let _ = event_tx_clone.send(HeosEvent::Disconnected).await;
                        break;
                    }
                    Ok(_) => {
                        if let Some(response) = Self::parse_response(&line) {
                            if response.is_event() {
                                Self::handle_event(&response, &event_tx_clone).await;
                            } else {
                                let _ = event_tx_clone.send(HeosEvent::Response(response)).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx_clone
                            .send(HeosEvent::Error(format!("Read error: {}", e)))
                            .await;
                        break;
                    }
                }
            }

            *write_half_clone.lock().await = None;
        });

        // Spawn writer task
        let write_half_for_writer = write_half.clone();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let mut guard = write_half_for_writer.lock().await;
                if let Some(writer) = guard.as_mut() {
                    let cmd_str = cmd.to_string();
                    if writer.write_all(cmd_str.as_bytes()).await.is_err() {
                        break;
                    }
                    if writer.flush().await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        event_tx.send(HeosEvent::Connected).await?;

        Ok(HeosHandle { cmd_tx })
    }

    fn parse_response(line: &str) -> Option<HeosResponse> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        serde_json::from_str(trimmed).ok()
    }

    async fn handle_event(response: &HeosResponse, tx: &mpsc::Sender<HeosEvent>) {
        let command = &response.heos.command;
        let params = response.parse_message();

        let event = match command.as_str() {
            protocol::EVENT_PLAYER_STATE_CHANGED => {
                let pid = params.get("pid").and_then(|s| s.parse().ok()).unwrap_or(0);
                let state = params
                    .get("state")
                    .map(|s| PlayState::from_str(s))
                    .unwrap_or_default();
                Some(HeosEvent::PlayerStateChanged { pid, state })
            }
            protocol::EVENT_PLAYER_NOW_PLAYING_CHANGED => {
                let pid = params.get("pid").and_then(|s| s.parse().ok()).unwrap_or(0);
                Some(HeosEvent::NowPlayingChanged { pid })
            }
            protocol::EVENT_PLAYER_VOLUME_CHANGED => {
                let pid = params.get("pid").and_then(|s| s.parse().ok()).unwrap_or(0);
                let level = params.get("level").and_then(|s| s.parse().ok()).unwrap_or(0);
                let mute = params
                    .get("mute")
                    .map(|s| MuteState::from_str(s))
                    .unwrap_or_default();
                Some(HeosEvent::VolumeChanged { pid, level, mute })
            }
            protocol::EVENT_REPEAT_MODE_CHANGED | protocol::EVENT_SHUFFLE_MODE_CHANGED => {
                let pid = params.get("pid").and_then(|s| s.parse().ok()).unwrap_or(0);
                let repeat = params
                    .get("repeat")
                    .map(|s| RepeatMode::from_str(s))
                    .unwrap_or_default();
                let shuffle = params
                    .get("shuffle")
                    .map(|s| ShuffleMode::from_str(s))
                    .unwrap_or_default();
                Some(HeosEvent::PlayModeChanged { pid, repeat, shuffle })
            }
            protocol::EVENT_PLAYER_QUEUE_CHANGED => {
                let pid = params.get("pid").and_then(|s| s.parse().ok()).unwrap_or(0);
                Some(HeosEvent::QueueChanged { pid })
            }
            protocol::EVENT_PLAYERS_CHANGED => {
                Some(HeosEvent::PlayersChanged(Vec::new()))
            }
            _ => None,
        };

        if let Some(event) = event {
            let _ = tx.send(event).await;
        }
    }
}
