use std::error::Error;
use std::process::{Command};

use std::env;
use std::process::exit;

use crate::ui;
use crate::error;

pub enum Program {
    Ffmpeg,
    Ffprobe
}

// returns the path of either ffmpeg or ffprobe
fn program_path(program: Program) -> String {
    // get the path of the current executable and remove the actual executable from the path so we just get the directory it's in
    let mut path = env::current_exe().expect("could not get path of the current executable");
    path.pop();

    // convert program enum to program string
    let program_str = match program {
        Program::Ffmpeg => "ffmpeg".to_owned(),
        Program::Ffprobe => "ffprobe".to_owned()
    };

    // add either "ffmpeg.exe" or just "ffmpeg" depending on the OS
    if cfg!(windows) {
        path.push(program_str + ".exe");
    }
    else {
        path.push(program_str);
    }

    if !path.exists() {
        error::show_error(format!("failed to find \"{}\", are you sure it's in the same directory as icecutter?", path.display()));
        exit(1);
    }

    path.to_string_lossy().to_string()
}

// returns the ffmpeg version
// this can theoretically panic however by the time this function is called we've already established that we have a valid ffmpeg/ffprobe installation
pub fn program_version(program: Program) -> String {
    let command = Command::new(program_path(program))
        .arg("-version")
        .output()
        .expect("ffmpeg/ffprobe has to be valid and installed correctly");

    let output = String::from_utf8(command.stdout).expect("output has to be valid utf8");
    let output: Vec<&str> = output.split(' ').collect();

    output[2].to_owned()
}

// returns the length of the video in MM:SS format
pub fn video_length(video: &str) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get video length in HOURS:MM:SS.MICROSECONDS format
    let output = Command::new(program_path(Program::Ffprobe))
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

    if output.len() < 3 {
        return Err(format!("failed to get duration of video, is the input file '{video}' valid?").into());
    }

    let minutes = &output[1];
    let seconds = &output[2];

    Ok(format!("{minutes}:{seconds}"))
}

// returns the FPS of the video
pub fn video_fps(video: &str) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get FPS in FPS/1 format
    let output = Command::new(program_path(Program::Ffprobe))
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

    // get output from command
    let output = String::from_utf8(output.stdout)?;
    let output: Vec<&str> = output.split('/').collect();

    // get the first number and the second number
    let x = output[0].trim().parse::<f32>()?;
    let y = output[1].trim().parse::<f32>()?;

    // divide the two numbers together, giving us the FPS
    Ok((x / y).round().to_string())
}

// converts the video via fffmpeg
// expects sanitized input (correct from/to timestamps, valid FPS, etc.)
pub fn convert(state: &ui::State, output: &str) -> Result<(), Box<dyn Error>> {
    // arg vec that we will push arguments into depending on the configuration
    let mut arguments: Vec<&str> = Vec::new();

    // input file
    arguments.push("-i");
    arguments.push(&state.file);

    // cutting
    if !state.from.is_empty() && !state.to.is_empty() {
        arguments.extend_from_slice(&["-ss", &state.from, "-to", &state.to]);
    }

    // conversion to 720p
    if state.convert_720p {
        arguments.extend_from_slice(&["-vf", "scale=-1:720"]);
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
    Command::new(program_path(Program::Ffmpeg))
        .args(&arguments)
        .spawn()?;

    println!("{:?}", arguments);

    Ok(())
}