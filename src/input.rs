use std::collections::HashSet;

use rdev::{Event, EventType, Key};

use crate::keystrokes::{Keystroke, Modifiers};

#[derive(Debug, Clone)]
pub enum GlobalInputEvent {
    Event(Event),
    ListenerFailed(String),
}

#[derive(Debug)]
pub enum InputEvent {
    Keystroke(Keystroke),
    ModifiersChanged(Modifiers),
}

/// A stateful normalizer for global input events.
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
            EventType::KeyPress(key) => match key.is_modifier() {
                true => self.update_modifiers(key, true),
                false => Some(InputEvent::Keystroke(Keystroke {
                    key: key.into(),
                    modifiers: self.modifiers,
                    text: event.into_printable_name(),
                })),
            },
            EventType::KeyRelease(key) => match key.is_modifier() {
                true => self.update_modifiers(key, false),
                false => None,
            },
            _ => None,
        }
    }

    fn update_modifiers(&mut self, key: Key, pressed: bool) -> Option<InputEvent> {
        if pressed {
            self.pressed_modifier_keys.insert(key);
        } else {
            self.pressed_modifier_keys.remove(&key);
        }

        let modifiers = Modifiers {
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

        let old_modifiers = std::mem::replace(&mut self.modifiers, modifiers);
        if self.modifiers != old_modifiers {
            Some(InputEvent::ModifiersChanged(self.modifiers))
        } else {
            None
        }
    }
}

/// Listener for global input events.
pub fn listener() -> impl iced::futures::Stream<Item = GlobalInputEvent> {
    use iced::futures::SinkExt;

    iced::stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<GlobalInputEvent>| async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<GlobalInputEvent>();

            std::thread::spawn(move || {
                let tx_clone = tx.clone();

                #[cfg(target_os = "macos")]
                rdev::set_is_main_thread(false);

                if let Err(e) = rdev::listen(move |event| {
                    let _ = tx_clone.send(GlobalInputEvent::Event(event));
                }) {
                    let _ = tx.send(GlobalInputEvent::ListenerFailed(format!("{e:?}")));
                }
            });

            while let Some(event) = rx.recv().await {
                let _ = output.send(event).await;
            }
        },
    )
}
