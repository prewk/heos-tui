use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub pid: i64,
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub network: String,
    #[serde(default)]
    pub lineout: i32,
    #[serde(default)]
    pub serial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NowPlayingMedia {
    #[serde(default)]
    pub song: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub mid: String,
    #[serde(default)]
    pub qid: i64,
    #[serde(default)]
    pub sid: i64,
    #[serde(default)]
    pub station: String,
    #[serde(rename = "type", default)]
    pub media_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayState {
    #[default]
    Unknown,
    Play,
    Pause,
    Stop,
}

impl PlayState {
    pub fn from_str(s: &str) -> Self {
        match s {
            "play" => PlayState::Play,
            "pause" => PlayState::Pause,
            "stop" => PlayState::Stop,
            _ => PlayState::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PlayState::Unknown => "unknown",
            PlayState::Play => "play",
            PlayState::Pause => "pause",
            PlayState::Stop => "stop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MuteState {
    #[default]
    Off,
    On,
}

impl MuteState {
    pub fn from_str(s: &str) -> Self {
        match s {
            "on" => MuteState::On,
            _ => MuteState::Off,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MuteState::On => "on",
            MuteState::Off => "off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    OnOne,
    OnAll,
}

impl RepeatMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "on_one" => RepeatMode::OnOne,
            "on_all" => RepeatMode::OnAll,
            _ => RepeatMode::Off,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RepeatMode::Off => "off",
            RepeatMode::OnOne => "on_one",
            RepeatMode::OnAll => "on_all",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::OnAll,
            RepeatMode::OnAll => RepeatMode::OnOne,
            RepeatMode::OnOne => RepeatMode::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShuffleMode {
    #[default]
    Off,
    On,
}

impl ShuffleMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "on" => ShuffleMode::On,
            _ => ShuffleMode::Off,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ShuffleMode::On => "on",
            ShuffleMode::Off => "off",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            ShuffleMode::Off => ShuffleMode::On,
            ShuffleMode::On => ShuffleMode::Off,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub qid: i64,
    pub song: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub mid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicSource {
    pub sid: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub available: String,
    #[serde(default)]
    pub service_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseItem {
    #[serde(default)]
    pub container: String,
    #[serde(default)]
    pub cid: String,
    #[serde(default)]
    pub mid: String,
    pub name: String,
    #[serde(rename = "type", default)]
    pub item_type: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub playable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSource {
    pub sid: i64,
    pub name: String,
    #[serde(default)]
    pub input: String,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub player: Option<Player>,
    pub now_playing: NowPlayingMedia,
    pub play_state: PlayState,
    pub volume: u8,
    pub mute: MuteState,
    pub repeat: RepeatMode,
    pub shuffle: ShuffleMode,
}
