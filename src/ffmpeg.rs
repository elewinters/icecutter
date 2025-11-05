use std::error::Error;
use std::process::{Command, Stdio};
use std::env;

use crate::ui;

fn program_path(program: &str, installed: bool) -> String {
    // if ffmpeg is installed we just return "ffmpeg"
    if installed {
        return String::from(program);
    }

    // get the path of the current executable and remove the actual executable from the path so we just get the directory it's in
    let mut exe_dir = env::current_exe().unwrap();
    exe_dir.pop();

    // add either "ffmpeg.exe" or just "ffmpeg" depending on the OS
    if cfg!(windows) {
        exe_dir.push(program.to_string() + ".exe");
    }
    else {
        exe_dir.push(program);
    }

    return exe_dir.to_str().unwrap().to_string();
}

// check if specified program is properly installed on the system
pub fn check_program(program: &str) -> Result<(), String> {
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
pub fn video_length(video: &str, ffprobe_installed: bool) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get video length in HOURS:MM:SS.MICROSECONDS format
    let output = Command::new(program_path("ffprobe", ffprobe_installed))
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
pub fn video_fps(video: &str, ffprobe_installed: bool) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get FPS in FPS/1 format
    let output = Command::new(program_path("ffprobe", ffprobe_installed))
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
    Command::new(program_path("ffmpeg", state.ffmpeg_installed))
        .args(&arguments)
        .spawn()?;

    println!("{:?}", arguments);

    Ok(())
}