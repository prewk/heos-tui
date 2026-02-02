use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    tx: mpsc::Sender<AppEvent>,
    rx: mpsc::Receiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let event_tx = tx.clone();

        std::thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if event_tx.blocking_send(AppEvent::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if event_tx.blocking_send(AppEvent::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else {
                    if event_tx.blocking_send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { tx, rx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::Sender<AppEvent> {
        self.tx.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    PlayPause,
    Stop,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    ToggleMute,
    CycleRepeat,
    ToggleShuffle,
    ShowDevices,
    ShowQueue,
    ShowBrowse,
    ShowInputs,
    ShowSurroundModes,
    ShowSoundSettings,
    ShowHelp,
    Back,
    Select,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Refresh,
}

impl Action {
    pub fn from_key(key: KeyEvent) -> Option<Self> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                Some(Action::Quit)
            }
            (KeyCode::Char(' '), _) | (KeyCode::Char('p'), _) => Some(Action::PlayPause),
            (KeyCode::Char('s'), _) => Some(Action::Stop),
            (KeyCode::Char('n'), _) | (KeyCode::Right, KeyModifiers::CONTROL) => {
                Some(Action::NextTrack)
            }
            (KeyCode::Char('b'), _) | (KeyCode::Left, KeyModifiers::CONTROL) => {
                Some(Action::PrevTrack)
            }
            (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => Some(Action::VolumeUp),
            (KeyCode::Char('-'), _) => Some(Action::VolumeDown),
            (KeyCode::Char('m'), _) => Some(Action::ToggleMute),
            (KeyCode::Char('r'), _) => Some(Action::CycleRepeat),
            (KeyCode::Char('z'), _) => Some(Action::ToggleShuffle),
            (KeyCode::Char('d'), _) => Some(Action::ShowDevices),
            (KeyCode::Char('u'), _) => Some(Action::ShowQueue),
            (KeyCode::Char('o'), _) => Some(Action::ShowBrowse),
            (KeyCode::Char('i'), _) => Some(Action::ShowInputs),
            (KeyCode::Char('a'), _) => Some(Action::ShowSurroundModes),
            (KeyCode::Char('w'), _) => Some(Action::ShowSoundSettings),
            (KeyCode::Char('?'), _) | (KeyCode::F(1), _) => Some(Action::ShowHelp),
            (KeyCode::Esc, _) => Some(Action::Back),
            (KeyCode::Enter, _) => Some(Action::Select),
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(Action::MoveUp),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(Action::MoveDown),
            (KeyCode::Left, _) | (KeyCode::Char('h'), _) => Some(Action::MoveLeft),
            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => Some(Action::MoveRight),
            (KeyCode::F(5), _) => Some(Action::Refresh),
            _ => None,
        }
    }
}
