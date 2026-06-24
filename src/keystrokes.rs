use std::borrow::Cow;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use rdev::Key;

use crate::icons::Icon;

const INACTIVITY_TIMEOUT: Duration = Duration::from_millis(1000);
const BUBBLE_TTL: Duration = Duration::from_millis(5000);
pub const MAX_ACTIVE_TEXT_LEN: usize = 24;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub fn any(&self) -> bool {
        self.control || self.alt || self.shift || self.meta
    }

    /// Returns true if any shortcut-like modifier is active.
    pub fn enables_shortcut(&self) -> bool {
        self.control || self.alt || self.meta
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKey {
    Char(char),
    Backspace,
    Delete,
    Escape,
    Enter,
    Tab,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    CapsLock,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    Function,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    BrightnessUp,
    BrightnessDown,
    PreviousTrack,
    PlayPause,
    PlayCd,
    NextTrack,
    F(u8),
    Unknown,
}

impl DisplayKey {
    fn is_delimiter(&self) -> bool {
        matches!(self, Self::Space | Self::Enter | Self::Tab)
    }

    fn is_special(&self) -> bool {
        !matches!(self, Self::Char(_))
    }
}

impl From<Key> for DisplayKey {
    fn from(key: Key) -> Self {
        match key {
            Key::Backspace => Self::Backspace,
            Key::CapsLock => Self::CapsLock,
            Key::Delete => Self::Delete,
            Key::DownArrow => Self::ArrowDown,
            Key::End => Self::End,
            Key::Escape => Self::Escape,
            Key::F1 => Self::F(1),
            Key::F2 => Self::F(2),
            Key::F3 => Self::F(3),
            Key::F4 => Self::F(4),
            Key::F5 => Self::F(5),
            Key::F6 => Self::F(6),
            Key::F7 => Self::F(7),
            Key::F8 => Self::F(8),
            Key::F9 => Self::F(9),
            Key::F10 => Self::F(10),
            Key::F11 => Self::F(11),
            Key::F12 => Self::F(12),
            Key::F13 => Self::F(13),
            Key::F14 => Self::F(14),
            Key::F15 => Self::F(15),
            Key::F16 => Self::F(16),
            Key::F17 => Self::F(17),
            Key::F18 => Self::F(18),
            Key::F19 => Self::F(19),
            Key::F20 => Self::F(20),
            Key::F21 => Self::F(21),
            Key::F22 => Self::F(22),
            Key::F23 => Self::F(23),
            Key::F24 => Self::F(24),
            Key::Home => Self::Home,
            Key::LeftArrow => Self::ArrowLeft,
            Key::PageDown => Self::PageDown,
            Key::PageUp => Self::PageUp,
            Key::Return | Key::KpReturn => Self::Enter,
            Key::RightArrow => Self::ArrowRight,
            Key::Space => Self::Space,
            Key::Tab => Self::Tab,
            Key::UpArrow => Self::ArrowUp,
            Key::PrintScreen => Self::PrintScreen,
            Key::ScrollLock => Self::ScrollLock,
            Key::Pause => Self::Pause,
            Key::NumLock => Self::NumLock,
            Key::BackQuote => Self::Char('`'),
            Key::Num1 | Key::Kp1 => Self::Char('1'),
            Key::Num2 | Key::Kp2 => Self::Char('2'),
            Key::Num3 | Key::Kp3 => Self::Char('3'),
            Key::Num4 | Key::Kp4 => Self::Char('4'),
            Key::Num5 | Key::Kp5 => Self::Char('5'),
            Key::Num6 | Key::Kp6 => Self::Char('6'),
            Key::Num7 | Key::Kp7 => Self::Char('7'),
            Key::Num8 | Key::Kp8 => Self::Char('8'),
            Key::Num9 | Key::Kp9 => Self::Char('9'),
            Key::Num0 | Key::Kp0 => Self::Char('0'),
            Key::Minus | Key::KpMinus => Self::Char('-'),
            Key::Equal => Self::Char('='),
            Key::KeyQ => Self::Char('Q'),
            Key::KeyW => Self::Char('W'),
            Key::KeyE => Self::Char('E'),
            Key::KeyR => Self::Char('R'),
            Key::KeyT => Self::Char('T'),
            Key::KeyY => Self::Char('Y'),
            Key::KeyU => Self::Char('U'),
            Key::KeyI => Self::Char('I'),
            Key::KeyO => Self::Char('O'),
            Key::KeyP => Self::Char('P'),
            Key::LeftBracket => Self::Char('['),
            Key::RightBracket => Self::Char(']'),
            Key::KeyA => Self::Char('A'),
            Key::KeyS => Self::Char('S'),
            Key::KeyD => Self::Char('D'),
            Key::KeyF => Self::Char('F'),
            Key::KeyG => Self::Char('G'),
            Key::KeyH => Self::Char('H'),
            Key::KeyJ => Self::Char('J'),
            Key::KeyK => Self::Char('K'),
            Key::KeyL => Self::Char('L'),
            Key::SemiColon => Self::Char(';'),
            Key::Quote => Self::Char('\''),
            Key::BackSlash | Key::IntlBackslash => Self::Char('\\'),
            Key::KeyZ => Self::Char('Z'),
            Key::KeyX => Self::Char('X'),
            Key::KeyC => Self::Char('C'),
            Key::KeyV => Self::Char('V'),
            Key::KeyB => Self::Char('B'),
            Key::KeyN => Self::Char('N'),
            Key::KeyM => Self::Char('M'),
            Key::Comma => Self::Char(','),
            Key::Dot | Key::KpDelete => Self::Char('.'),
            Key::Slash | Key::KpDivide => Self::Char('/'),
            Key::Insert => Self::Insert,
            Key::KpPlus => Self::Char('+'),
            Key::KpMultiply => Self::Char('*'),
            Key::Function => Self::Function,
            Key::VolumeUp => Self::VolumeUp,
            Key::VolumeDown => Self::VolumeDown,
            Key::VolumeMute => Self::VolumeMute,
            Key::BrightnessUp => Self::BrightnessUp,
            Key::BrightnessDown => Self::BrightnessDown,
            Key::PreviousTrack => Self::PreviousTrack,
            Key::PlayPause => Self::PlayPause,
            Key::PlayCd => Self::PlayCd,
            Key::NextTrack => Self::NextTrack,
            Key::Alt
            | Key::AltGr
            | Key::ControlLeft
            | Key::ControlRight
            | Key::MetaLeft
            | Key::MetaRight
            | Key::ShiftLeft
            | Key::ShiftRight
            | Key::Unknown(_) => Self::Unknown,
        }
    }
}

/// A visual label rendered for a keystroke.
#[derive(Debug)]
pub enum KeyLabel<'a> {
    Text(Cow<'a, str>),
    Char(char),
    Icon(Icon),
}

impl From<DisplayKey> for KeyLabel<'static> {
    fn from(key: DisplayKey) -> Self {
        match key {
            DisplayKey::Char(ch) => Self::Char(ch),
            DisplayKey::Backspace => Self::Icon(Icon::Delete),
            DisplayKey::Delete => Self::Icon(Icon::DeleteRev),
            DisplayKey::Escape => Self::Icon(Icon::CircleArrowOutUpLeft),
            DisplayKey::Enter => Self::Icon(Icon::CornerDownLeft),
            DisplayKey::Tab => Self::Icon(Icon::ArrowRightToLine),
            DisplayKey::Space => Self::Icon(Icon::SpaceNarrow),
            DisplayKey::ArrowUp => Self::Icon(Icon::ArrowUp),
            DisplayKey::ArrowDown => Self::Icon(Icon::ArrowDown),
            DisplayKey::ArrowLeft => Self::Icon(Icon::ArrowLeft),
            DisplayKey::ArrowRight => Self::Icon(Icon::ArrowRight),
            DisplayKey::Home => Self::Icon(Icon::ArrowUpLeft),
            DisplayKey::End => Self::Icon(Icon::ArrowDownRight),
            DisplayKey::PageUp => Self::Icon(Icon::ArrowUpToLine),
            DisplayKey::PageDown => Self::Icon(Icon::ArrowDownToLine),
            DisplayKey::Insert => Self::Icon(Icon::Insert),
            DisplayKey::CapsLock => Self::Icon(Icon::ArrowBigUpDash),
            DisplayKey::PrintScreen => Self::Text(Cow::Borrowed("PrtSc")),
            DisplayKey::ScrollLock => Self::Text(Cow::Borrowed("ScrLk")),
            DisplayKey::Pause => Self::Icon(Icon::Pause),
            DisplayKey::NumLock => Self::Text(Cow::Borrowed("NumLk")),
            DisplayKey::Function => Self::Text(Cow::Borrowed("Fn")),
            DisplayKey::VolumeUp => Self::Icon(Icon::Volume2),
            DisplayKey::VolumeDown => Self::Icon(Icon::Volume1),
            DisplayKey::VolumeMute => Self::Icon(Icon::VolumeOff),
            DisplayKey::BrightnessUp => Self::Icon(Icon::Sun),
            DisplayKey::BrightnessDown => Self::Icon(Icon::SunDim),
            DisplayKey::PreviousTrack => Self::Icon(Icon::SkipBack),
            DisplayKey::PlayPause => Self::Icon(Icon::PlayPause),
            DisplayKey::PlayCd => Self::Icon(Icon::Disc3),
            DisplayKey::NextTrack => Self::Icon(Icon::SkipForward),
            DisplayKey::F(x) => Self::Text(Cow::Owned(format!("F{x}"))),
            DisplayKey::Unknown => Self::Text(Cow::Borrowed("Unknown")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Keystroke {
    pub key: DisplayKey,
    pub modifiers: Modifiers,
    /// Display text for normal typing.
    ///
    /// This is layout-resolved and could be `None` for non-printable keys, in
    /// which case [`Keystroke::key`] should be used as the fallback.
    pub text: Option<String>,
}

impl Keystroke {
    pub fn labels(&self) -> Vec<KeyLabel<'static>> {
        let mut parts = Vec::new();

        if self.modifiers.control {
            parts.push(KeyLabel::Icon(Icon::ChevronUp));
        }
        if self.modifiers.alt {
            parts.push(KeyLabel::Icon(Icon::Option));
        }
        if self.modifiers.shift {
            parts.push(KeyLabel::Icon(Icon::ArrowBigUp));
        }
        if self.modifiers.meta {
            parts.push(KeyLabel::Icon(Icon::Command));
        }

        parts.push(self.key.into());
        parts
    }
}

#[derive(Debug)]
pub struct KeystrokeState {
    pub active: String,
    pub history: VecDeque<KeyBubble>,
    history_limit: usize,
    last_key_at: Option<Instant>,
}

impl KeystrokeState {
    pub fn new(history_limit: usize) -> Self {
        Self {
            active: String::new(),
            history: VecDeque::new(),
            history_limit,
            last_key_at: None,
        }
    }

    pub fn set_history_limit(&mut self, history_limit: usize) {
        self.history_limit = history_limit;
        self.trim_history();
    }

    pub fn handle_keystroke(&mut self, keystroke: Keystroke, now: Instant) {
        self.finalize_if_inactive(now);
        self.last_key_at = Some(now);

        if keystroke.key.is_delimiter() {
            if keystroke.modifiers.any() {
                // Modified delimiters are explicit key events
                self.finalize_active(now);
                let kind = if keystroke.modifiers.enables_shortcut() {
                    BubbleKind::Shortcut
                } else {
                    BubbleKind::Special
                };
                self.push_history(vec![BubblePart::Key(keystroke)], kind, now);
            } else {
                // Plain delimiters belong to the typing row that they end
                let mut parts = Vec::new();
                if !self.active.is_empty() {
                    parts.push(BubblePart::Text(std::mem::take(&mut self.active)));
                }
                parts.push(BubblePart::Key(keystroke));
                self.push_history(parts, BubbleKind::Typing, now);
                self.last_key_at = None;
            }
            return;
        }

        if keystroke.modifiers.enables_shortcut() {
            // Shortcut-like chords are keystrokes first, not typed text, so
            // that e.g. Ctrl+Shift+7 will be resolved as the physical shortcut
            // rather than as a layout-resolved character like Ctrl+&
            self.finalize_active(now);
            self.push_history(vec![BubblePart::Key(keystroke)], BubbleKind::Shortcut, now);
            return;
        }

        if let Some(text) = &keystroke.text {
            for ch in text.chars() {
                if self.active.len() >= MAX_ACTIVE_TEXT_LEN {
                    self.finalize_active(now);
                }
                self.active.push(ch);
            }
            if !self.active.is_empty() {
                self.last_key_at = Some(now);
            }
            return;
        }

        if keystroke.key.is_special() {
            self.finalize_active(now);
            self.push_history(vec![BubblePart::Key(keystroke)], BubbleKind::Special, now);
        }
    }

    /// Finalize the active typing row if there has been a pause.
    pub fn finalize_if_inactive(&mut self, now: Instant) {
        if self
            .last_key_at
            .is_some_and(|last_key_at| now.duration_since(last_key_at) >= INACTIVITY_TIMEOUT)
        {
            self.finalize_active(now);
        }
    }

    /// Remove expired rows from history.
    pub fn prune_expired(&mut self, now: Instant) {
        while self
            .history
            .front()
            .is_some_and(|bubble| now.duration_since(bubble.last_updated_at) >= BUBBLE_TTL)
        {
            self.history.pop_front();
        }
    }

    /// Finalize the active typing row by moving it to history and clearing it.
    fn finalize_active(&mut self, now: Instant) {
        if !self.active.is_empty() {
            let text = std::mem::take(&mut self.active);
            self.push_history(vec![BubblePart::Text(text)], BubbleKind::Typing, now);
        }
        self.last_key_at = None;
    }

    /// Push a new row to history.
    ///
    /// If the new row is identical to the most recent one and both are
    /// key-only, they will be merged into a single row with an incremented
    /// repeat count.
    fn push_history(&mut self, parts: Vec<BubblePart>, kind: BubbleKind, now: Instant) {
        if parts.iter().all(|part| matches!(part, BubblePart::Key(_)))
            && let Some(last) = self.history.back_mut()
            && last.kind == kind
            && last.parts == parts
        {
            last.count += 1;
            last.last_updated_at = now;
            return;
        }

        self.history.push_back(KeyBubble {
            parts,
            kind,
            count: 1,
            last_updated_at: now,
        });

        self.trim_history();
    }

    /// Trim history to fit within the history limit.
    fn trim_history(&mut self) {
        while self.history.len() > self.history_limit {
            self.history.pop_front();
        }
    }
}

/// One finalized row in the visible keystroke history.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyBubble {
    pub parts: Vec<BubblePart>,
    pub kind: BubbleKind,
    pub count: usize,
    last_updated_at: Instant,
}

/// A piece of content inside a finalized keystroke row.
#[derive(Debug, PartialEq, Eq)]
pub enum BubblePart {
    Text(String),
    Key(Keystroke),
}

/// The visual category for a finalized keystroke row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleKind {
    Typing,
    Shortcut,
    Special,
}
