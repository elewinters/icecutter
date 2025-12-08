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
use std::time::Duration;

use iced::window::Settings;
use iced::Task;

mod ui;
mod error;
mod ffmpeg;

// accepts a time stamp in MM:SS format and returns the amount of seconds it represents
pub fn timestamp_to_secs(timestamp: &str) -> f32 {
    let split: Vec<&str> = timestamp.split(':').collect();

    let minutes = split[0].parse::<u64>()
        .expect("we have already established that the timestamp is valid");
    let seconds = split[1].parse::<f32>()
        .expect("we have already established that the timestamp is valid");

    (Duration::from_mins(minutes) + Duration::from_secs_f32(seconds)).as_secs_f32()
}

fn main() {
    // get input video file command line argument, this can be None
    let file: Option<String> = env::args().nth(1);

    // initialize state if file argument exists
    // if initialization fails, fall back to default state
    let state = match file {
        Some(f) => ui::initialize_state(Path::new(&f)).unwrap_or_else(|err| {
            error!("failed to initialize state: {err}");
            ui::State::default()
        }),
        None => ui::State::default()
    };

    // run iced application with custom initial state
    let result = iced::application("icecutter", ui::State::update, ui::State::view)
        .subscription(ui::State::subscription)
        .window(Settings {
            size: (640.0, 480.0).into(),
            resizable: false,
            ..Default::default()
        })
        .run_with(move || (state, Task::none()));
    
    if let Err(err) = result {
        error!("failed to initialize iced, something must've went very wrong: {err}");
    }
}