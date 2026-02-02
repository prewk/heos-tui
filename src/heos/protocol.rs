use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeosResponse {
    pub heos: HeosHeader,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub options: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeosHeader {
    pub command: String,
    pub result: Option<String>,
    #[serde(default)]
    pub message: String,
}

impl HeosResponse {
    pub fn is_success(&self) -> bool {
        self.heos.result.as_deref() == Some("success")
    }

    pub fn is_event(&self) -> bool {
        self.heos.result.is_none()
    }

    pub fn parse_message(&self) -> HashMap<String, String> {
        parse_message_string(&self.heos.message)
    }

    pub fn get_payload_array<T: for<'de> Deserialize<'de>>(&self) -> Option<Vec<T>> {
        serde_json::from_value(self.payload.clone()).ok()
    }

    pub fn get_payload_object<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_value(self.payload.clone()).ok()
    }
}

pub fn parse_message_string(message: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if message.is_empty() {
        return map;
    }
    for pair in message.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

#[derive(Debug, Clone)]
pub struct HeosCommand {
    pub group: &'static str,
    pub command: &'static str,
    pub params: Vec<(String, String)>,
}

impl HeosCommand {
    pub fn new(group: &'static str, command: &'static str) -> Self {
        Self {
            group,
            command,
            params: Vec::new(),
        }
    }

    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.push((key.into(), value.into()));
        self
    }

    pub fn to_string(&self) -> String {
        let mut cmd = format!("heos://{}/{}", self.group, self.command);
        if !self.params.is_empty() {
            cmd.push('?');
            let params: Vec<String> = self
                .params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            cmd.push_str(&params.join("&"));
        }
        cmd.push_str("\r\n");
        cmd
    }
}

// System commands
pub fn register_for_change_events(enable: bool) -> HeosCommand {
    HeosCommand::new("system", "register_for_change_events")
        .param("enable", if enable { "on" } else { "off" })
}

pub fn check_account() -> HeosCommand {
    HeosCommand::new("system", "check_account")
}

pub fn heart_beat() -> HeosCommand {
    HeosCommand::new("system", "heart_beat")
}

// Player commands
pub fn get_players() -> HeosCommand {
    HeosCommand::new("player", "get_players")
}

pub fn get_player_info(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_player_info").param("pid", pid.to_string())
}

pub fn get_play_state(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_play_state").param("pid", pid.to_string())
}

pub fn set_play_state(pid: i64, state: &str) -> HeosCommand {
    HeosCommand::new("player", "set_play_state")
        .param("pid", pid.to_string())
        .param("state", state)
}

pub fn get_now_playing_media(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_now_playing_media").param("pid", pid.to_string())
}

pub fn get_volume(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_volume").param("pid", pid.to_string())
}

pub fn set_volume(pid: i64, level: u8) -> HeosCommand {
    HeosCommand::new("player", "set_volume")
        .param("pid", pid.to_string())
        .param("level", level.to_string())
}

pub fn volume_up(pid: i64, step: u8) -> HeosCommand {
    HeosCommand::new("player", "volume_up")
        .param("pid", pid.to_string())
        .param("step", step.to_string())
}

pub fn volume_down(pid: i64, step: u8) -> HeosCommand {
    HeosCommand::new("player", "volume_down")
        .param("pid", pid.to_string())
        .param("step", step.to_string())
}

pub fn get_mute(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_mute").param("pid", pid.to_string())
}

pub fn set_mute(pid: i64, state: &str) -> HeosCommand {
    HeosCommand::new("player", "set_mute")
        .param("pid", pid.to_string())
        .param("state", state)
}

pub fn toggle_mute(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "toggle_mute").param("pid", pid.to_string())
}

pub fn get_play_mode(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "get_play_mode").param("pid", pid.to_string())
}

pub fn set_play_mode(pid: i64, repeat: &str, shuffle: &str) -> HeosCommand {
    HeosCommand::new("player", "set_play_mode")
        .param("pid", pid.to_string())
        .param("repeat", repeat)
        .param("shuffle", shuffle)
}

pub fn get_queue(pid: i64, start: u32, end: u32) -> HeosCommand {
    HeosCommand::new("player", "get_queue")
        .param("pid", pid.to_string())
        .param("range", format!("{},{}", start, end))
}

pub fn play_queue(pid: i64, qid: i64) -> HeosCommand {
    HeosCommand::new("player", "play_queue")
        .param("pid", pid.to_string())
        .param("qid", qid.to_string())
}

pub fn remove_from_queue(pid: i64, qid: i64) -> HeosCommand {
    HeosCommand::new("player", "remove_from_queue")
        .param("pid", pid.to_string())
        .param("qid", qid.to_string())
}

pub fn clear_queue(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "clear_queue").param("pid", pid.to_string())
}

pub fn play_next(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "play_next").param("pid", pid.to_string())
}

pub fn play_previous(pid: i64) -> HeosCommand {
    HeosCommand::new("player", "play_previous").param("pid", pid.to_string())
}

// Browse commands
pub fn get_music_sources() -> HeosCommand {
    HeosCommand::new("browse", "get_music_sources")
}

pub fn get_source_info(sid: i64) -> HeosCommand {
    HeosCommand::new("browse", "get_source_info").param("sid", sid.to_string())
}

pub fn browse_source(sid: i64) -> HeosCommand {
    HeosCommand::new("browse", "browse").param("sid", sid.to_string())
}

pub fn browse_source_container(sid: i64, cid: &str) -> HeosCommand {
    HeosCommand::new("browse", "browse")
        .param("sid", sid.to_string())
        .param("cid", cid)
}

pub fn play_station(pid: i64, sid: i64, mid: &str) -> HeosCommand {
    HeosCommand::new("browse", "play_stream")
        .param("pid", pid.to_string())
        .param("sid", sid.to_string())
        .param("mid", mid)
}

pub fn play_input(pid: i64, input: &str) -> HeosCommand {
    HeosCommand::new("browse", "play_input")
        .param("pid", pid.to_string())
        .param("input", input)
}

pub fn play_input_source(pid: i64, spid: i64, input: &str) -> HeosCommand {
    HeosCommand::new("browse", "play_input")
        .param("pid", pid.to_string())
        .param("spid", spid.to_string())
        .param("input", input)
}

// Event names
pub const EVENT_PLAYER_STATE_CHANGED: &str = "event/player_state_changed";
pub const EVENT_PLAYER_NOW_PLAYING_CHANGED: &str = "event/player_now_playing_changed";
pub const EVENT_PLAYER_NOW_PLAYING_PROGRESS: &str = "event/player_now_playing_progress";
pub const EVENT_PLAYER_VOLUME_CHANGED: &str = "event/player_volume_changed";
pub const EVENT_PLAYER_PLAYBACK_ERROR: &str = "event/player_playback_error";
pub const EVENT_PLAYER_QUEUE_CHANGED: &str = "event/player_queue_changed";
pub const EVENT_REPEAT_MODE_CHANGED: &str = "event/repeat_mode_changed";
pub const EVENT_SHUFFLE_MODE_CHANGED: &str = "event/shuffle_mode_changed";
pub const EVENT_PLAYERS_CHANGED: &str = "event/players_changed";
pub const EVENT_GROUPS_CHANGED: &str = "event/groups_changed";
pub const EVENT_SOURCES_CHANGED: &str = "event/sources_changed";
