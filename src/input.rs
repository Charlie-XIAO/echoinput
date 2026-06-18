use std::collections::HashSet;

use iced::futures::SinkExt;
use rdev::{Event, EventType, Key};

use crate::keystrokes::{KeyId, Keystroke, Modifiers};

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keystroke(Keystroke),
    ModifiersChanged(Modifiers),
}

#[derive(Debug, Default)]
pub struct InputNormalizer {
    modifiers: Modifiers,
    pressed_modifier_keys: HashSet<Key>,
}

impl InputNormalizer {
    pub fn reset_modifiers(&mut self) {
        self.modifiers = Modifiers::default();
        self.pressed_modifier_keys.clear();
    }

    pub fn handle_event(&mut self, event: Event) -> Option<InputEvent> {
        match event.event_type {
            EventType::KeyPress(key) => {
                if is_modifier_key(key) {
                    return self.update_modifier_key(key, true);
                }

                Some(InputEvent::Keystroke(Keystroke {
                    key: key_id_for_rdev_key(key),
                    modifiers: self.modifiers,
                    text: printable_text(event.name.as_deref()).map(str::to_string),
                }))
            },
            EventType::KeyRelease(key) => {
                if is_modifier_key(key) {
                    return self.update_modifier_key(key, false);
                }

                None
            },
            _ => None,
        }
    }

    fn update_modifier_key(&mut self, key: Key, pressed: bool) -> Option<InputEvent> {
        let previous = self.modifiers;

        if pressed {
            self.pressed_modifier_keys.insert(key);
        } else {
            self.pressed_modifier_keys.remove(&key);
        }

        self.modifiers = Modifiers {
            control: self
                .pressed_modifier_keys
                .iter()
                .any(|key| matches!(key, Key::ControlLeft | Key::ControlRight)),
            alt: self
                .pressed_modifier_keys
                .iter()
                .any(|key| matches!(key, Key::Alt | Key::AltGr)),
            shift: self
                .pressed_modifier_keys
                .iter()
                .any(|key| matches!(key, Key::ShiftLeft | Key::ShiftRight)),
            meta: self
                .pressed_modifier_keys
                .iter()
                .any(|key| matches!(key, Key::MetaLeft | Key::MetaRight)),
        };

        if self.modifiers != previous {
            Some(InputEvent::ModifiersChanged(self.modifiers))
        } else {
            None
        }
    }
}

pub fn global_input_listener() -> impl iced::futures::Stream<Item = Event> {
    iced::stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<Event>| async move {
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Event>();

            std::thread::spawn(move || {
                if let Err(err) = rdev::listen(move |event| {
                    let _ = sender.send(event);
                }) {
                    eprintln!("Failed to start global input listener: {:?}", err);
                }
            });

            while let Some(event) = receiver.recv().await {
                let _ = output.send(event).await;
            }
        },
    )
}

fn is_modifier_key(key: Key) -> bool {
    matches!(
        key,
        Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::ShiftLeft
            | Key::ShiftRight
            | Key::MetaLeft
            | Key::MetaRight
    )
}

fn printable_text(event_name: Option<&str>) -> Option<&str> {
    let name = event_name?;

    if name
        .chars()
        .all(|character| !character.is_control() && !character.is_whitespace())
    {
        Some(name)
    } else {
        None
    }
}

fn key_id_for_rdev_key(key: Key) -> KeyId {
    match key {
        Key::Backspace => KeyId::Backspace,
        Key::Delete => KeyId::Delete,
        Key::DownArrow => KeyId::ArrowDown,
        Key::End => KeyId::End,
        Key::Escape => KeyId::Escape,
        Key::F1 => KeyId::F(1),
        Key::F2 => KeyId::F(2),
        Key::F3 => KeyId::F(3),
        Key::F4 => KeyId::F(4),
        Key::F5 => KeyId::F(5),
        Key::F6 => KeyId::F(6),
        Key::F7 => KeyId::F(7),
        Key::F8 => KeyId::F(8),
        Key::F9 => KeyId::F(9),
        Key::F10 => KeyId::F(10),
        Key::F11 => KeyId::F(11),
        Key::F12 => KeyId::F(12),
        Key::Home => KeyId::Home,
        Key::LeftArrow => KeyId::ArrowLeft,
        Key::PageDown => KeyId::PageDown,
        Key::PageUp => KeyId::PageUp,
        Key::Return | Key::KpReturn => KeyId::Enter,
        Key::RightArrow => KeyId::ArrowRight,
        Key::Space => KeyId::Space,
        Key::Tab => KeyId::Tab,
        Key::UpArrow => KeyId::ArrowUp,
        Key::BackQuote => KeyId::Character('`'),
        Key::Num1 | Key::Kp1 => KeyId::Character('1'),
        Key::Num2 | Key::Kp2 => KeyId::Character('2'),
        Key::Num3 | Key::Kp3 => KeyId::Character('3'),
        Key::Num4 | Key::Kp4 => KeyId::Character('4'),
        Key::Num5 | Key::Kp5 => KeyId::Character('5'),
        Key::Num6 | Key::Kp6 => KeyId::Character('6'),
        Key::Num7 | Key::Kp7 => KeyId::Character('7'),
        Key::Num8 | Key::Kp8 => KeyId::Character('8'),
        Key::Num9 | Key::Kp9 => KeyId::Character('9'),
        Key::Num0 | Key::Kp0 => KeyId::Character('0'),
        Key::Minus | Key::KpMinus => KeyId::Character('-'),
        Key::Equal => KeyId::Character('='),
        Key::KeyQ => KeyId::Character('Q'),
        Key::KeyW => KeyId::Character('W'),
        Key::KeyE => KeyId::Character('E'),
        Key::KeyR => KeyId::Character('R'),
        Key::KeyT => KeyId::Character('T'),
        Key::KeyY => KeyId::Character('Y'),
        Key::KeyU => KeyId::Character('U'),
        Key::KeyI => KeyId::Character('I'),
        Key::KeyO => KeyId::Character('O'),
        Key::KeyP => KeyId::Character('P'),
        Key::LeftBracket => KeyId::Character('['),
        Key::RightBracket => KeyId::Character(']'),
        Key::KeyA => KeyId::Character('A'),
        Key::KeyS => KeyId::Character('S'),
        Key::KeyD => KeyId::Character('D'),
        Key::KeyF => KeyId::Character('F'),
        Key::KeyG => KeyId::Character('G'),
        Key::KeyH => KeyId::Character('H'),
        Key::KeyJ => KeyId::Character('J'),
        Key::KeyK => KeyId::Character('K'),
        Key::KeyL => KeyId::Character('L'),
        Key::SemiColon => KeyId::Character(';'),
        Key::Quote => KeyId::Character('\''),
        Key::BackSlash | Key::IntlBackslash => KeyId::Character('\\'),
        Key::KeyZ => KeyId::Character('Z'),
        Key::KeyX => KeyId::Character('X'),
        Key::KeyC => KeyId::Character('C'),
        Key::KeyV => KeyId::Character('V'),
        Key::KeyB => KeyId::Character('B'),
        Key::KeyN => KeyId::Character('N'),
        Key::KeyM => KeyId::Character('M'),
        Key::Comma => KeyId::Character(','),
        Key::Dot | Key::KpDelete => KeyId::Character('.'),
        Key::Slash | Key::KpDivide => KeyId::Character('/'),
        Key::KpPlus => KeyId::Character('+'),
        Key::KpMultiply => KeyId::Character('*'),
        Key::Insert => KeyId::Insert,
        Key::CapsLock => KeyId::CapsLock,
        Key::PrintScreen => KeyId::PrintScreen,
        Key::ScrollLock => KeyId::ScrollLock,
        Key::Pause => KeyId::Pause,
        Key::NumLock => KeyId::NumLock,
        Key::Function => KeyId::Function,
        _ => KeyId::Unknown,
    }
}
