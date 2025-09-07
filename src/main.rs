use std::env;

use std::error::Error;
use std::process::{Command, Stdio};

mod ui;

// check if specified program is properly installed on the system
fn check_program(program: &str) -> Result<(), String> {
    let command = Command::new(program)
        .arg("-version")
        .stdout(Stdio::null()) // dont print ffmpeg/ffprobe output to console
        .status();
    
    let err = Err(format!("failed to detect {program} on this system, are you sure it's been installed correctly?"));

    match command {
        Ok(status) if !status.success() => err,
        Err(_) => err,
        _ => Ok(())
    }
}

// returns the length of the video in MM:SS format
fn video_length(video: &str) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get video length in HOURS:MM:SS.MICROSECONDS format
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

    // check if success
    if !output.status.success() {
        return Err(format!("ffprobe command failed, is the input file '{video}' valid?").into());
    }

    // get output from command and split by : and . so that we can get only the minutes and seconds
    let output = String::from_utf8(output.stdout)?;
    let output: Vec<&str> = output.split(&[':', '.']).collect();

    let minutes = &output[1];
    let seconds = &output[2];

    Ok(format!("{minutes}:{seconds}"))
}

// returns the FPS of the video
fn video_fps(video: &str) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get FPS in FPS/1 format
    let output = Command::new("ffprobe")
        .arg("-i")
        .arg(video)
        .arg("-select_streams")
        .arg("v")
        .arg("-show_entries")
        .arg("stream=r_frame_rate")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0")
        .output()?;

    // check if success
    if !output.status.success() {
        return Err(format!("ffprobe command failed, is the input file '{video}' valid?").into());
    }

    // get output from command and split by / so we only get the actual fps
    let output = String::from_utf8(output.stdout)?;
    let output: Vec<&str> = output.split('/').collect();

    Ok(String::from(output[0]))
}

// converts the video via fffmpeg
// expects sanitized input (correct from/to timestamps, valid FPS, etc.)
pub fn convert(state: &ui::State, output: &str) -> Result<(), Box<dyn Error>> {
    // arg vec that we will push arguments into depending on the configuration
    let mut arguments: Vec<&str> = Vec::new();

    // argument arrays
    let convert_720p = ["-vf", "scale=-1:720"];
    let cut = ["-ss", &state.from, "-to", &state.to];

    // input file
    arguments.push("-i");
    arguments.push(&state.file);

    // cutting
    if !state.from.is_empty() && !state.to.is_empty() {
        arguments.extend_from_slice(&cut);
    }

    // conversion to 720p
    if state.convert_720p {
        arguments.extend_from_slice(&convert_720p);
    }

    // encode as h265
    arguments.push("-vcodec");
    arguments.push("libx265");

    // set fps
    arguments.push("-r");
    arguments.push(&state.fps);

    // set file size limit to 8M (setting this to 10M instead makes the output go above 10M sometimes, so we set this a bit lower to be more conservative) 
    arguments.push("-fs");
    arguments.push("8M");

    // overwrite files
    arguments.push("-y");

    // output file name
    arguments.push(output);

    // run command
    Command::new("ffmpeg")
        .args(&arguments)
        .spawn()?;

    println!("{:?}", arguments);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // check if required programs are installed
    check_program("ffmpeg")?;
    check_program("ffprobe")?;

    // get input video file command line argument, this can be None
    let file: Option<String> = env::args().nth(1);

    // get length and FPS of video if the file argument exists
    let length: Option<String> = file.as_ref().map(|f| video_length(f)).transpose()?;
    let fps: Option<String> = file.as_ref().map(|f| video_fps(f)).transpose()?;

    // run iced application with custom initial state
    iced::application("icecutter", ui::State::update, ui::State::view)
        .window(iced::window::Settings {
            size: (640.0, 480.0).into(),
            resizable: false,
            ..Default::default()
        })
        .run_with(|| {
            (
                ui::State {
                    // set "from" string to 00:00 if length is valid
                    from: length.as_ref().map_or(String::default(), |_| String::from("00:00")),
                    to: length.unwrap_or_default(),

                    file: file.unwrap_or_default(),
                    fps: fps.unwrap_or_default(),
                    ..Default::default()
                },
                iced::Task::none()
            )
        })?;
    
    Ok(())
}