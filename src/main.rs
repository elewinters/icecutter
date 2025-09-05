use std::env;
use std::io;

use std::error::Error;
use std::process::{Command, Stdio};

mod ui;

// runs to check if this program is properly installed on the system
fn check_program(program: &str) -> Result<(), String> {
    let command = Command::new(program)
        .arg("-version")
        .stdout(Stdio::null()) // dont print ffmpeg/ffprobe output to console
        .status();
    
    let err = Err(format!("failed to detect {program} on this system, are you sure it's been installed correctly?"));

    match command {
        Ok(status) if !status.success() => return err,
        Err(_) => return err,
        _ => Ok(())
    }
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

    check_program("ffmpeg")?;
    check_program("ffprobe")?;

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