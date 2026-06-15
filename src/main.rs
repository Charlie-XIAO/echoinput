mod fonts;
mod input;
mod keystrokes;
mod ui;

use std::time::{Duration, Instant};

use iced::theme::Base;
use iced::window::{self, Level, Position};
use iced::{Color, Element, Point, Size, Subscription, Task, Theme};
use rdev::Event;

use crate::fonts::ICON_FONT_DATA;
use crate::input::{InputEvent, InputNormalizer};
use crate::keystrokes::{KeystrokeState, Modifiers};
use crate::ui::OverlayLayout;

const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn main() -> iced::Result {
    let layout = OverlayLayout::default();

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
        .window(overlay_window_settings(layout))
        .run()
}

fn overlay_window_settings(layout: OverlayLayout) -> window::Settings {
    let window_size = layout.window_size();
    let settings = window::Settings {
        size: window_size,
        position: Position::Specific(Point::new(layout.screen_margin, layout.screen_margin)),
        transparent: true,
        decorations: false,
        level: Level::AlwaysOnTop,
        resizable: false,
        ..Default::default()
    };

    #[cfg(target_os = "windows")]
    let settings = {
        let mut settings = settings;
        settings.platform_specific.skip_taskbar = true;
        settings
    };

    settings
}

#[derive(Debug, Default)]
struct App {
    window_id: Option<window::Id>,
    layout: OverlayLayout,
    input: InputNormalizer,
    keystrokes: KeystrokeState,
    held_modifiers: Modifiers,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Id, window::Event),
    MonitorSize(window::Id, Option<Size>),
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
                    return Task::batch(vec![
                        window::enable_mouse_passthrough(id),
                        window::monitor_size(id)
                            .map(move |monitor_size| Message::MonitorSize(id, monitor_size)),
                    ]);
                }

                Task::none()
            },
            Message::MonitorSize(id, Some(monitor_size)) => {
                let window_size = clamped_window_size(
                    self.layout.window_size(),
                    monitor_size,
                    self.layout.screen_margin,
                );
                let position =
                    bottom_left_position(window_size, monitor_size, self.layout.screen_margin);

                Task::batch(vec![
                    window::resize(id, window_size),
                    window::move_to(id, position),
                ])
            },
            Message::MonitorSize(_, None) => Task::none(),
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
        ui::overlay(&self.keystrokes, self.held_modifiers, self.layout)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::events().map(|(id, event)| Message::WindowEvent(id, event)),
            Subscription::run(input::global_input_listener).map(Message::InputHookEvent),
            iced::time::every(TICK_INTERVAL).map(Message::Tick),
        ])
    }
}

fn clamped_window_size(desired: Size, monitor_size: Size, screen_margin: f32) -> Size {
    let max_width = (monitor_size.width - screen_margin * 2.0).max(1.0);
    let max_height = (monitor_size.height - screen_margin * 2.0).max(1.0);

    Size::new(desired.width.min(max_width), desired.height.min(max_height))
}

fn bottom_left_position(window_size: Size, monitor_size: Size, screen_margin: f32) -> Point {
    Point::new(
        screen_margin,
        (monitor_size.height - window_size.height - screen_margin).max(screen_margin),
    )
}
