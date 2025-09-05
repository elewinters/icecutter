use std::env;
use std::process::{exit, Command, Stdio};

use iced::*;

mod ui;

// runs to check if ffmpeg is properly installed on the system
fn check_ffmpeg() -> bool {
    let command = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null()) // dont print ffmpeg output to console
        .status();
    
    let status = match command {
        Ok(status) => status,
        Err(_) => return false,
    };

    status.success()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = args.get(1).cloned();

    if !check_ffmpeg() {
        println!("failed to detect ffmpeg on this system");
        exit(1);
    }

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
        .unwrap();
}