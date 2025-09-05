use std::env;
use iced::*;

mod ui;

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    let file = args.get(1).cloned();

    let window_settings = window::Settings {
        size: (640.0, 480.0).into(),
        resizable: false,
        ..Default::default()
    };

    iced::application("icecutter", ui::State::update, ui::State::view)
        .window(window_settings)
        .run_with(|| {
            (
                ui::State {
                    file: file.unwrap_or_default(),
                    ..Default::default()
                },
                Task::none()
            )
        })
}