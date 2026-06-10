use iced::window::{self, Level};
use iced::{Background, Color, Element, Subscription, Task, Theme};
use iced::widget::container;
use iced::futures::SinkExt;

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .theme(|_: &App| Theme::Dark)
        .title(|_: &App| String::from("EchoInput"))
        .subscription(App::subscription)
        .window(window::Settings {
            transparent: true,
            decorations: false,
            level: Level::AlwaysOnTop,
            fullscreen: true,
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Default)]
struct App {
    window_id: Option<window::Id>,
    passthrough_enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Id, window::Event),
    InputHookEvent(rdev::Event),
}

impl App {
    fn boot() -> Self {
        Self {
            window_id: None,
            passthrough_enabled: false,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowEvent(id, _event) => {
                if self.window_id.is_none() {
                    self.window_id = Some(id);
                    self.passthrough_enabled = true;
                    return window::enable_mouse_passthrough(id);
                }
                Task::none()
            }
            Message::InputHookEvent(event) => {
                println!("Captured global event: {:?}", event);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container("")
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::events().map(|(id, event)| Message::WindowEvent(id, event)),
            Subscription::run(global_input_listener).map(Message::InputHookEvent),
        ])
    }
}

fn global_input_listener() -> impl iced::futures::Stream<Item = rdev::Event> {
    iced::stream::channel(100, |mut output: iced::futures::channel::mpsc::Sender<rdev::Event>| async move {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<rdev::Event>();

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
    })
}
