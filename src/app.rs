use std::time::{Duration, Instant};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::theme::Base;
use iced::{Color, Element, Subscription, Task, Theme, window};

use crate::hotkey::HotKeyId;
use crate::icons::ICON_FONT_DATA;
use crate::input::{GlobalInputEvent, InputEvent, InputNormalizer};
use crate::keystrokes::{KeystrokeState, Modifiers};
use crate::settings::Settings;
use crate::ui::Layout;
use crate::window::Geometry;

const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn run() -> iced::Result {
    iced::daemon(App::boot, App::update, App::view)
        .title("EchoInput")
        .theme(Theme::Dark)
        .style(|_: &App, theme: &Theme| {
            let mut style = theme.base();
            style.background_color = Color::TRANSPARENT;
            style
        })
        .font(ICON_FONT_DATA)
        .subscription(App::subscription)
        .run()
}

struct App {
    window_id: window::Id,
    settings: Settings,
    layout: Layout,
    geometry: Geometry,
    input: InputNormalizer,
    keystrokes: KeystrokeState,
    held_modifiers: Modifiers,
    _hotkey_manager: Option<GlobalHotKeyManager>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    MonitorSize(window::Id, Option<iced::Size>),
    InputEvent(GlobalInputEvent),
    HotkeyEvent(GlobalHotKeyEvent),
    Tick(Instant),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        if let Err(e) = crate::logging::init() {
            eprintln!("failed to initialize logging: {e:#}");
        }

        let hotkey_manager = match crate::hotkey::init() {
            Ok(manager) => Some(manager),
            Err(e) => {
                log::error!("failed to initialize hotkeys: {e:#}");
                None
            },
        };

        let settings = match crate::settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                log::error!("failed to load settings: {e:#}; using defaults");
                Settings::default()
            },
        };

        let layout = Layout::default();
        let geometry = Geometry::default();
        let keystrokes = KeystrokeState::new(settings.history_limit);

        let window_settings =
            crate::window::settings(layout.content_size(settings.history_limit), &geometry);
        let (window_id, open_window) = window::open(window_settings);

        (
            Self {
                window_id,
                settings,
                layout,
                geometry,
                input: InputNormalizer::default(),
                keystrokes,
                held_modifiers: Modifiers::default(),
                _hotkey_manager: hotkey_manager,
            },
            open_window.map(Message::WindowOpened),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => Task::batch(vec![
                window::enable_mouse_passthrough(id),
                window::monitor_size(id).map(move |size| Message::MonitorSize(id, size)),
                #[cfg(target_os = "linux")]
                crate::window::configure_x11_window(id),
            ]),
            Message::WindowClosed(id) => {
                assert_eq!(id, self.window_id, "unexpected window id: {id}");
                iced::exit()
            },
            Message::MonitorSize(id, Some(monitor_size)) => {
                let (size, position) = crate::window::placement(
                    self.layout.content_size(self.settings.history_limit),
                    monitor_size,
                    &self.geometry,
                );
                Task::batch(vec![
                    window::resize(id, size),
                    window::move_to(id, position),
                ])
            },
            Message::MonitorSize(_, None) => {
                log::warn!("failed to get monitor size");
                Task::none()
            },
            Message::InputEvent(event) => {
                let event = match event {
                    GlobalInputEvent::Event(event) => event,
                    GlobalInputEvent::ListenerFailed(e) => {
                        log::error!("failed to start global input listener: {e}");
                        return Task::none();
                    },
                };

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
            Message::HotkeyEvent(event) => {
                if event.state != HotKeyState::Released {
                    return Task::none();
                }

                // We know the hotkey is released, so we reset the modifiers to
                // avoid displaying stuck modifiers
                self.input.reset_modifiers();
                self.held_modifiers = Modifiers::default();

                match event.id {
                    HotKeyId::OPEN_SETTINGS => {
                        if let Err(e) = crate::settings::open() {
                            log::error!("failed to open settings file: {e:#}");
                        }
                        Task::none()
                    },
                    HotKeyId::RELOAD_SETTINGS => match crate::settings::load() {
                        Ok(settings) => {
                            log::info!("settings reloaded");
                            self.apply_settings(settings)
                        },
                        Err(e) => {
                            log::warn!("failed to reload settings: {e:#}");
                            Task::none()
                        },
                    },
                    HotKeyId::OPEN_LOG => {
                        if let Err(e) = crate::logging::open() {
                            log::warn!("failed to open log file: {e:#}");
                        }
                        Task::none()
                    },
                    HotKeyId::QUIT => iced::exit(),
                    _ => {
                        log::warn!("unrecognized hotkey event: {}", event.id);
                        Task::none()
                    },
                }
            },
            Message::Tick(now) => {
                self.keystrokes.finalize_if_inactive(now);
                self.keystrokes.prune_expired(now);
                Task::none()
            },
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        assert_eq!(window, self.window_id, "unexpected window id: {window}");
        self.layout.view(&self.keystrokes, &self.held_modifiers)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::close_events().map(Message::WindowClosed),
            Subscription::run(crate::input::listener).map(Message::InputEvent),
            Subscription::run(crate::hotkey::listener).map(Message::HotkeyEvent),
            iced::time::every(TICK_INTERVAL).map(Message::Tick),
        ])
    }

    fn apply_settings(&mut self, settings: Settings) -> Task<Message> {
        let old_settings = std::mem::replace(&mut self.settings, settings);

        if self.settings.history_limit != old_settings.history_limit {
            self.keystrokes
                .set_history_limit(self.settings.history_limit);
            let window_id = self.window_id;
            return window::monitor_size(window_id)
                .map(move |monitor_size| Message::MonitorSize(window_id, monitor_size));
        }

        Task::none()
    }
}
