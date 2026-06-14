mod fonts;
mod input;
mod keystrokes;
mod ui;

use std::time::{Duration, Instant};

use iced::theme::Base;
use iced::window::{self, Level};
use iced::{Color, Element, Subscription, Task, Theme};
use rdev::Event;

use crate::fonts::ICON_FONT_DATA;
use crate::input::{InputEvent, InputNormalizer};
use crate::keystrokes::{KeystrokeState, Modifiers};

const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .theme(|_: &App| Theme::Dark)
        .style(|_: &App, theme: &Theme| {
            let mut style = theme.base();
            style.background_color = Color::TRANSPARENT;
            style
        })
        .font(ICON_FONT_DATA)
        .title(|_: &App| String::from("EchoInput"))
        .subscription(App::subscription)
        .window(overlay_window_settings())
        .run()
}

fn overlay_window_settings() -> window::Settings {
    let mut settings = window::Settings {
        transparent: true,
        decorations: false,
        level: Level::AlwaysOnTop,
        fullscreen: true,
        ..Default::default()
    };

    #[cfg(target_os = "windows")]
    {
        settings.platform_specific.skip_taskbar = true;
    }

    settings
}

#[derive(Debug, Default)]
struct App {
    window_id: Option<window::Id>,
    input: InputNormalizer,
    keystrokes: KeystrokeState,
    held_modifiers: Modifiers,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Id, window::Event),
    InputHookEvent(Event),
    Tick(Instant),
}

impl App {
    fn boot() -> Self {
        Self::default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowEvent(id, _event) => {
                if self.window_id.is_none() {
                    self.window_id = Some(id);
                    return window::enable_mouse_passthrough(id);
                }

                Task::none()
            },
            Message::InputHookEvent(event) => {
                match self.input.handle_event(event) {
                    Some(InputEvent::Keystroke(keystroke)) => {
                        self.keystrokes.handle_keystroke(keystroke, Instant::now());
                    },
                    Some(InputEvent::ModifiersChanged(modifiers)) => {
                        self.held_modifiers = modifiers;
                    },
                    None => {},
                }

                Task::none()
            },
            Message::Tick(now) => {
                self.keystrokes.finalize_if_inactive(now);
                self.keystrokes.prune_expired(now);
                Task::none()
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        ui::overlay(&self.keystrokes, self.held_modifiers)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::events().map(|(id, event)| Message::WindowEvent(id, event)),
            Subscription::run(input::global_input_listener).map(Message::InputHookEvent),
            iced::time::every(TICK_INTERVAL).map(Message::Tick),
        ])
    }
}
