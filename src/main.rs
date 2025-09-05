use std::env;
use std::io;

use std::error::Error;
use std::process::{Command, Stdio};

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

fn video_length(video: &str) -> Result<String, io::Error> {
    let output = Command::new("ffprobe")
        .arg("-i")
        .arg(video)
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0")
        .arg("-sexagesimal")
        .output()?;

    // get output from command and split by : and . so that we can get only the minutes and seconds
    let output = String::from_utf8(output.stdout).unwrap();
    let output: Vec<&str> = output.split(&[':', '.']).collect();

    let minutes = &output[1];
    let seconds = &output[2];

    Ok(format!("{minutes}:{seconds}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file = args.get(1).cloned();

    if !check_ffmpeg() {
        return Err("failed to detect ffmpeg on this system".into());
    }

    let length = match file {
        Some(ref file) => Some(video_length(file)?),
        None => None,
    };

    let window_settings = iced::window::Settings {
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

                    from: if length.is_some() {
                        String::from("00:00")
                    }
                    else {
                        Default::default()
                    },

                    to: length.unwrap_or_default(),
                    
                    ..Default::default()
                },
                iced::Task::none()
            )
        })?;
    
    Ok(())
}