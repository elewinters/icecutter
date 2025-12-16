// disable console window on windows, but not in debug mode, cuz for some reason printing doesn't work right with this on
#![cfg_attr(
    all(
        target_os = "windows",
        not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]

use std::env;
use std::path::Path;

use iced::window::Settings;

mod ui;
mod error;
mod ffmpeg;
mod timestamp;

use ui::State;

fn main() {
    // get input video file command line argument, this can be None
    let file: Option<String> = env::args().nth(1);

    // initialize state if file argument exists
    // if initialization fails, fall back to default state
    let state = match file {
        Some(f) => State::new(Path::new(&f)).unwrap_or_else(|err| {
            error!("failed to initialize state: {err}");
            State::default()
        }),
        None => State::default()
    };

    // run iced application with custom initial state
    let result = iced::application(move || state.clone(), State::update, State::view)
        .title("icecutter")
        .window(Settings {
            size: (640.0, 500.0).into(),
            resizable: false,
            ..Default::default()
        })
        .subscription(State::subscription)
        .run();
    
    if let Err(err) = result {
        error!("failed to initialize iced, something must've went very wrong: {err}");
    }
}