use iced::*;

mod ui;

fn main() -> iced::Result {

    let window_settings = window::Settings {
        size: (640.0, 480.0).into(),
        resizable: false,
        ..Default::default()
    };

    iced::application("icecutter", ui::State::update, ui::State::view)
        .window(window_settings)
        .run_with(|| {
            (ui::State::default(), Task::none())
        })
}