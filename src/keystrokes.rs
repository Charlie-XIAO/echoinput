use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::fonts::{
    ICON_KBD_ALT, ICON_KBD_BACKSPACE, ICON_KBD_CONTROL, ICON_KBD_DELETE, ICON_KBD_DOWN,
    ICON_KBD_END, ICON_KBD_ESCAPE, ICON_KBD_HOME, ICON_KBD_LEFT, ICON_KBD_META, ICON_KBD_PAGEDOWN,
    ICON_KBD_PAGEUP, ICON_KBD_RETURN, ICON_KBD_RIGHT, ICON_KBD_SHIFT, ICON_KBD_SPACE, ICON_KBD_TAB,
    ICON_KBD_UP,
};

pub const INACTIVITY_TIMEOUT: Duration = Duration::from_millis(1000);
pub const BUBBLE_TTL: Duration = Duration::from_millis(5000);
pub const DEFAULT_HISTORY_LIMIT: usize = 5;
pub const MAX_ACTIVE_TEXT_LEN: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keystroke {
    pub key: KeyId,
    pub modifiers: Modifiers,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyId {
    Character(char),
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
    F(u8),
    Unknown,
}

impl KeyId {
    fn is_delimiter(self) -> bool {
        matches!(self, Self::Space | Self::Enter | Self::Tab)
    }

    fn is_special(self) -> bool {
        !matches!(self, Self::Character(_))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub fn any(self) -> bool {
        self.control || self.alt || self.shift || self.meta
    }

    pub fn has_command_modifier(self) -> bool {
        self.control || self.alt || self.meta
    }
}

#[derive(Debug)]
pub struct KeystrokeState {
    pub active: String,
    pub history: VecDeque<KeyBubble>,
    history_limit: usize,
    last_key_at: Option<Instant>,
}

impl Default for KeystrokeState {
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_LIMIT)
    }
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
                self.finalize_active(now);
                let kind = if keystroke.modifiers.has_command_modifier() {
                    BubbleKind::Shortcut
                } else {
                    BubbleKind::Special
                };
                self.push_history(vec![BubblePart::Key(keystroke)], kind, now);
            } else {
                self.finalize_active_with_suffix(BubblePart::Key(keystroke), now);
            }

            return;
        }

        if keystroke.modifiers.has_command_modifier() {
            self.finalize_active(now);
            self.push_history(vec![BubblePart::Key(keystroke)], BubbleKind::Shortcut, now);
            return;
        }

        if let Some(text) = &keystroke.text {
            self.append_active_text(text, now);

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

    pub fn finalize_if_inactive(&mut self, now: Instant) {
        if self
            .last_key_at
            .is_some_and(|last_key_at| now.duration_since(last_key_at) >= INACTIVITY_TIMEOUT)
        {
            self.finalize_active(now);
        }
    }

    pub fn prune_expired(&mut self, now: Instant) {
        while self
            .history
            .front()
            .is_some_and(|bubble| now.duration_since(bubble.last_updated_at) >= BUBBLE_TTL)
        {
            self.history.pop_front();
        }
    }

    fn finalize_active(&mut self, now: Instant) {
        if !self.active.is_empty() {
            let text = std::mem::take(&mut self.active);
            self.push_history(vec![BubblePart::Text(text)], BubbleKind::Typing, now);
        }

        self.last_key_at = None;
    }

    fn finalize_active_with_suffix(&mut self, suffix: BubblePart, now: Instant) {
        let mut parts = Vec::new();

        if !self.active.is_empty() {
            parts.push(BubblePart::Text(std::mem::take(&mut self.active)));
        }

        parts.push(suffix);
        self.push_history(parts, BubbleKind::Typing, now);
        self.last_key_at = None;
    }

    fn append_active_text(&mut self, text: &str, now: Instant) {
        for character in text.chars() {
            if self.active.chars().count() >= MAX_ACTIVE_TEXT_LEN {
                self.finalize_active(now);
            }
            self.active.push(character);
        }
    }

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

    fn trim_history(&mut self) {
        while self.history.len() > self.history_limit {
            self.history.pop_front();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBubble {
    pub parts: Vec<BubblePart>,
    pub kind: BubbleKind,
    pub count: usize,
    last_updated_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BubblePart {
    Text(String),
    Key(Keystroke),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleKind {
    Typing,
    Shortcut,
    Special,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLabelPart {
    Text(String),
    Icon(char),
}

pub fn key_label_parts(keystroke: &Keystroke) -> Vec<KeyLabelPart> {
    let mut parts = Vec::new();

    if keystroke.modifiers.control {
        parts.push(KeyLabelPart::Icon(ICON_KBD_CONTROL));
    }
    if keystroke.modifiers.alt {
        parts.push(KeyLabelPart::Icon(ICON_KBD_ALT));
    }
    if keystroke.modifiers.shift {
        parts.push(KeyLabelPart::Icon(ICON_KBD_SHIFT));
    }
    if keystroke.modifiers.meta {
        parts.push(KeyLabelPart::Icon(ICON_KBD_META));
    }

    parts.push(match keystroke.key {
        KeyId::Character(ch) => KeyLabelPart::Text(ch.to_string()),
        KeyId::Backspace => KeyLabelPart::Icon(ICON_KBD_BACKSPACE),
        KeyId::Delete => KeyLabelPart::Icon(ICON_KBD_DELETE),
        KeyId::Escape => KeyLabelPart::Icon(ICON_KBD_ESCAPE),
        KeyId::Enter => KeyLabelPart::Icon(ICON_KBD_RETURN),
        KeyId::Tab => KeyLabelPart::Icon(ICON_KBD_TAB),
        KeyId::Space => KeyLabelPart::Icon(ICON_KBD_SPACE),
        KeyId::ArrowUp => KeyLabelPart::Icon(ICON_KBD_UP),
        KeyId::ArrowDown => KeyLabelPart::Icon(ICON_KBD_DOWN),
        KeyId::ArrowLeft => KeyLabelPart::Icon(ICON_KBD_LEFT),
        KeyId::ArrowRight => KeyLabelPart::Icon(ICON_KBD_RIGHT),
        KeyId::Home => KeyLabelPart::Icon(ICON_KBD_HOME),
        KeyId::End => KeyLabelPart::Icon(ICON_KBD_END),
        KeyId::PageUp => KeyLabelPart::Icon(ICON_KBD_PAGEUP),
        KeyId::PageDown => KeyLabelPart::Icon(ICON_KBD_PAGEDOWN),
        KeyId::Insert => KeyLabelPart::Text(String::from("Ins")),
        KeyId::CapsLock => KeyLabelPart::Text(String::from("Caps")),
        KeyId::PrintScreen => KeyLabelPart::Text(String::from("Print")),
        KeyId::ScrollLock => KeyLabelPart::Text(String::from("Scroll")),
        KeyId::Pause => KeyLabelPart::Text(String::from("Pause")),
        KeyId::NumLock => KeyLabelPart::Text(String::from("Num")),
        KeyId::Function => KeyLabelPart::Text(String::from("Fn")),
        KeyId::F(x) => KeyLabelPart::Text(format!("F{x}")),
        KeyId::Unknown => KeyLabelPart::Text(String::from("Unknown")),
    });

    parts
}
