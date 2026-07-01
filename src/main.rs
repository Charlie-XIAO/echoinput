mod app;
mod icons;
mod input;
mod keystrokes;
mod logging;
mod settings;
mod tray;
mod ui;
mod window;

pub fn main() -> iced::Result {
    app::run()
}
