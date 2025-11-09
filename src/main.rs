use std::env;

mod ui;
mod error;
mod ffmpeg;

fn main() {
    // get input video file command line argument, this can be None
    let file: Option<String> = env::args().nth(1);
    let state = match file {
        Some(f) => ui::initialize_state(&f).unwrap_or_else(|err| {
            error::show_error(format!("failed to initialize state: {err}"));
            ui::State::default()
        }),
        None => ui::State::default()
    };

    // run iced application with custom initial state
    let result = iced::application("icecutter", ui::State::update, ui::State::view)
        .window(iced::window::Settings {
            size: (640.0, 480.0).into(),
            resizable: false,
            ..Default::default()
        })
        .run_with(move || (state, iced::Task::none()));
    
    if let Err(err) = result {
        error::show_error(format!("failed to initialize iced, something must've went very wrong: {err}"));
    }
}