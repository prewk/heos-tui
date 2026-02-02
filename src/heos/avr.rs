use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub const AVR_PORT: u16 = 23;

/// Events from the AVR control protocol
#[derive(Debug, Clone)]
pub enum AvrEvent {
    Connected,
    Disconnected,
    MasterVolume(u8),       // 0-98
    Mute(bool),
    Power(bool),
    SurroundMode(String),
    InputSource(String),
    Error(String),
    Response(String),
}

/// Surround modes available on Denon/Marantz AVRs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurroundMode {
    Movie,
    Music,
    Game,
    Direct,
    PureDirect,
    Stereo,
    Auto,
    DolbyDigital,
    DtsSurround,
    MultiChStereo,
    RockArena,
    JazzClub,
    MonoMovie,
    Matrix,
    VideoGame,
    Virtual,
}

impl SurroundMode {
    pub fn command(&self) -> &'static str {
        match self {
            SurroundMode::Movie => "MSMOVIE",
            SurroundMode::Music => "MSMUSIC",
            SurroundMode::Game => "MSGAME",
            SurroundMode::Direct => "MSDIRECT",
            SurroundMode::PureDirect => "MSPURE DIRECT",
            SurroundMode::Stereo => "MSSTEREO",
            SurroundMode::Auto => "MSAUTO",
            SurroundMode::DolbyDigital => "MSDOLBY DIGITAL",
            SurroundMode::DtsSurround => "MSDTS SURROUND",
            SurroundMode::MultiChStereo => "MSMCH STEREO",
            SurroundMode::RockArena => "MSROCK ARENA",
            SurroundMode::JazzClub => "MSJAZZ CLUB",
            SurroundMode::MonoMovie => "MSMONO MOVIE",
            SurroundMode::Matrix => "MSMATRIX",
            SurroundMode::VideoGame => "MSVIDEO GAME",
            SurroundMode::Virtual => "MSVIRTUAL",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SurroundMode::Movie => "Movie",
            SurroundMode::Music => "Music",
            SurroundMode::Game => "Game",
            SurroundMode::Direct => "Direct",
            SurroundMode::PureDirect => "Pure Direct",
            SurroundMode::Stereo => "Stereo",
            SurroundMode::Auto => "Auto",
            SurroundMode::DolbyDigital => "Dolby Digital",
            SurroundMode::DtsSurround => "DTS Surround",
            SurroundMode::MultiChStereo => "Multi Ch Stereo",
            SurroundMode::RockArena => "Rock Arena",
            SurroundMode::JazzClub => "Jazz Club",
            SurroundMode::MonoMovie => "Mono Movie",
            SurroundMode::Matrix => "Matrix",
            SurroundMode::VideoGame => "Video Game",
            SurroundMode::Virtual => "Virtual",
        }
    }

    pub fn all() -> &'static [SurroundMode] {
        &[
            SurroundMode::Movie,
            SurroundMode::Music,
            SurroundMode::Game,
            SurroundMode::Direct,
            SurroundMode::PureDirect,
            SurroundMode::Stereo,
            SurroundMode::Auto,
            SurroundMode::DolbyDigital,
            SurroundMode::DtsSurround,
            SurroundMode::MultiChStereo,
            SurroundMode::RockArena,
            SurroundMode::JazzClub,
            SurroundMode::MonoMovie,
            SurroundMode::Matrix,
            SurroundMode::VideoGame,
            SurroundMode::Virtual,
        ]
    }

    pub fn from_response(s: &str) -> Option<Self> {
        let s = s.trim().to_uppercase();
        match s.as_str() {
            "MOVIE" => Some(SurroundMode::Movie),
            "MUSIC" => Some(SurroundMode::Music),
            "GAME" => Some(SurroundMode::Game),
            "DIRECT" => Some(SurroundMode::Direct),
            "PURE DIRECT" => Some(SurroundMode::PureDirect),
            "STEREO" => Some(SurroundMode::Stereo),
            "AUTO" => Some(SurroundMode::Auto),
            "DOLBY DIGITAL" => Some(SurroundMode::DolbyDigital),
            "DTS SURROUND" => Some(SurroundMode::DtsSurround),
            "MCH STEREO" => Some(SurroundMode::MultiChStereo),
            "ROCK ARENA" => Some(SurroundMode::RockArena),
            "JAZZ CLUB" => Some(SurroundMode::JazzClub),
            "MONO MOVIE" => Some(SurroundMode::MonoMovie),
            "MATRIX" => Some(SurroundMode::Matrix),
            "VIDEO GAME" => Some(SurroundMode::VideoGame),
            "VIRTUAL" => Some(SurroundMode::Virtual),
            _ => None,
        }
    }
}

/// Quick select modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSelect {
    Quick1,
    Quick2,
    Quick3,
    Quick4,
    Quick5,
}

impl QuickSelect {
    pub fn command(&self) -> &'static str {
        match self {
            QuickSelect::Quick1 => "MSQUICK1",
            QuickSelect::Quick2 => "MSQUICK2",
            QuickSelect::Quick3 => "MSQUICK3",
            QuickSelect::Quick4 => "MSQUICK4",
            QuickSelect::Quick5 => "MSQUICK5",
        }
    }
}

/// Handle for sending commands to the AVR
#[derive(Clone)]
pub struct AvrHandle {
    cmd_tx: mpsc::Sender<String>,
}

impl AvrHandle {
    pub async fn send_raw(&self, cmd: &str) -> Result<()> {
        self.cmd_tx
            .send(format!("{}\r", cmd))
            .await
            .map_err(|_| anyhow::anyhow!("AVR disconnected"))
    }

    // Power control
    pub async fn power_on(&self) -> Result<()> {
        self.send_raw("PWON").await
    }

    pub async fn power_off(&self) -> Result<()> {
        self.send_raw("PWSTANDBY").await
    }

    pub async fn get_power(&self) -> Result<()> {
        self.send_raw("PW?").await
    }

    // Master volume (00-98, or UP/DOWN)
    pub async fn volume_up(&self) -> Result<()> {
        self.send_raw("MVUP").await
    }

    pub async fn volume_down(&self) -> Result<()> {
        self.send_raw("MVDOWN").await
    }

    pub async fn set_volume(&self, level: u8) -> Result<()> {
        let level = level.min(98);
        self.send_raw(&format!("MV{:02}", level)).await
    }

    pub async fn get_volume(&self) -> Result<()> {
        self.send_raw("MV?").await
    }

    // Mute
    pub async fn mute_on(&self) -> Result<()> {
        self.send_raw("MUON").await
    }

    pub async fn mute_off(&self) -> Result<()> {
        self.send_raw("MUOFF").await
    }

    pub async fn mute_toggle(&self) -> Result<()> {
        // AVR doesn't have toggle, we'd need to track state
        // For now just query
        self.send_raw("MU?").await
    }

    pub async fn get_mute(&self) -> Result<()> {
        self.send_raw("MU?").await
    }

    // Surround mode
    pub async fn set_surround_mode(&self, mode: SurroundMode) -> Result<()> {
        self.send_raw(mode.command()).await
    }

    pub async fn get_surround_mode(&self) -> Result<()> {
        self.send_raw("MS?").await
    }

    // Input source
    pub async fn set_input(&self, input: &str) -> Result<()> {
        self.send_raw(&format!("SI{}", input)).await
    }

    pub async fn get_input(&self) -> Result<()> {
        self.send_raw("SI?").await
    }

    // Common inputs
    pub async fn input_tv(&self) -> Result<()> {
        self.set_input("TV").await
    }
    pub async fn input_dvd(&self) -> Result<()> {
        self.set_input("DVD").await
    }
    pub async fn input_bluray(&self) -> Result<()> {
        self.set_input("BD").await
    }
    pub async fn input_game(&self) -> Result<()> {
        self.set_input("GAME").await
    }
    pub async fn input_media_player(&self) -> Result<()> {
        self.set_input("MPLAY").await
    }
    pub async fn input_cbl_sat(&self) -> Result<()> {
        self.set_input("SAT/CBL").await
    }
    pub async fn input_network(&self) -> Result<()> {
        self.set_input("NET").await
    }
    pub async fn input_bluetooth(&self) -> Result<()> {
        self.set_input("BT").await
    }
    pub async fn input_hdmi(&self, num: u8) -> Result<()> {
        self.set_input(&format!("HDMI{}", num.min(7))).await
    }

    // Tone control
    pub async fn bass_up(&self) -> Result<()> {
        self.send_raw("PSBAS UP").await
    }

    pub async fn bass_down(&self) -> Result<()> {
        self.send_raw("PSBAS DOWN").await
    }

    pub async fn treble_up(&self) -> Result<()> {
        self.send_raw("PSTRE UP").await
    }

    pub async fn treble_down(&self) -> Result<()> {
        self.send_raw("PSTRE DOWN").await
    }

    // Dynamic EQ
    pub async fn dynamic_eq_on(&self) -> Result<()> {
        self.send_raw("PSDYNEQ ON").await
    }

    pub async fn dynamic_eq_off(&self) -> Result<()> {
        self.send_raw("PSDYNEQ OFF").await
    }

    // Dialog Enhancer
    pub async fn dialog_enhancer(&self, level: u8) -> Result<()> {
        let level = level.min(6);
        if level == 0 {
            self.send_raw("PSDIL OFF").await
        } else {
            self.send_raw(&format!("PSDIL {:02}", level)).await
        }
    }

    // Subwoofer level adjust
    pub async fn subwoofer_up(&self) -> Result<()> {
        self.send_raw("PSSWL UP").await
    }

    pub async fn subwoofer_down(&self) -> Result<()> {
        self.send_raw("PSSWL DOWN").await
    }

    // LFE level
    pub async fn lfe_up(&self) -> Result<()> {
        self.send_raw("PSLFE UP").await
    }

    pub async fn lfe_down(&self) -> Result<()> {
        self.send_raw("PSLFE DOWN").await
    }

    // Cinema EQ
    pub async fn cinema_eq_on(&self) -> Result<()> {
        self.send_raw("PSCINEMA EQ.ON").await
    }

    pub async fn cinema_eq_off(&self) -> Result<()> {
        self.send_raw("PSCINEMA EQ.OFF").await
    }

    // Night mode / Dynamic Volume
    pub async fn dynamic_volume(&self, mode: &str) -> Result<()> {
        // OFF, LIT (Light), MED (Medium), HEV (Heavy)
        self.send_raw(&format!("PSDYNVOL {}", mode)).await
    }

    // Query all status
    pub async fn query_status(&self) -> Result<()> {
        self.send_raw("PW?").await?;
        self.send_raw("MV?").await?;
        self.send_raw("MU?").await?;
        self.send_raw("SI?").await?;
        self.send_raw("MS?").await?;
        Ok(())
    }
}

pub struct AvrClient;

impl AvrClient {
    pub async fn connect(host: &str, event_tx: mpsc::Sender<AvrEvent>) -> Result<AvrHandle> {
        let addr = format!("{}:{}", host, AVR_PORT);
        let stream = TcpStream::connect(&addr)
            .await
            .context("Failed to connect to AVR control port")?;

        let (read_half, write_half) = stream.into_split();
        let write_half = Arc::new(Mutex::new(Some(write_half)));

        // Create command channel
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(100);

        // Spawn reader task
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let _ = event_tx_clone.send(AvrEvent::Disconnected).await;
                        break;
                    }
                    Ok(_) => {
                        let response = line.trim();
                        if !response.is_empty() {
                            Self::handle_response(response, &event_tx_clone).await;
                        }
                    }
                    Err(e) => {
                        let _ = event_tx_clone
                            .send(AvrEvent::Error(format!("Read error: {}", e)))
                            .await;
                        break;
                    }
                }
            }
        });

        // Spawn writer task
        let write_half_for_writer = write_half.clone();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let mut guard = write_half_for_writer.lock().await;
                if let Some(writer) = guard.as_mut() {
                    if writer.write_all(cmd.as_bytes()).await.is_err() {
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

        event_tx.send(AvrEvent::Connected).await?;

        Ok(AvrHandle { cmd_tx })
    }

    async fn handle_response(response: &str, tx: &mpsc::Sender<AvrEvent>) {
        let event = if response.starts_with("MV") && !response.starts_with("MVMAX") {
            // Master volume response: MV50 or MV505 (50.5)
            let vol_str = &response[2..];
            if let Ok(vol) = vol_str.parse::<u8>() {
                Some(AvrEvent::MasterVolume(vol))
            } else if vol_str.len() == 3 {
                // Handle half-dB values like "505" = 50.5
                if let Ok(vol) = vol_str[..2].parse::<u8>() {
                    Some(AvrEvent::MasterVolume(vol))
                } else {
                    None
                }
            } else {
                None
            }
        } else if response.starts_with("MU") {
            match &response[2..] {
                "ON" => Some(AvrEvent::Mute(true)),
                "OFF" => Some(AvrEvent::Mute(false)),
                _ => None,
            }
        } else if response.starts_with("PW") {
            match &response[2..] {
                "ON" => Some(AvrEvent::Power(true)),
                "STANDBY" | "OFF" => Some(AvrEvent::Power(false)),
                _ => None,
            }
        } else if response.starts_with("SI") {
            Some(AvrEvent::InputSource(response[2..].to_string()))
        } else if response.starts_with("MS") {
            Some(AvrEvent::SurroundMode(response[2..].to_string()))
        } else {
            Some(AvrEvent::Response(response.to_string()))
        };

        if let Some(event) = event {
            let _ = tx.send(event).await;
        }
    }
}
