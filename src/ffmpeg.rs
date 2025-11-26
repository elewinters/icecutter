use std::io::{self, BufRead, BufReader};

use std::error::Error;
use std::process::{Child, Command, Stdio};

// we apply this to every Command we create (on windows) as to not create a console window
// not doing this causes flashing console windows to keep popping up every time an ffmpeg/ffprobe command is ran
// which is Less than ideal
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use std::env;
use std::process::exit;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::futures::StreamExt;
use iced::stream;

use crate::ui;
use crate::error;

use ui::Message;

pub enum Program {
    Ffmpeg,
    Ffprobe
}

pub enum ConversionInput {
    Start {
        state: ui::State,
        output_file: String
    }
}

// returns the path of either ffmpeg or ffprobe
// if a global installaton is found in the user's PATH variable, it takes priority and gets returned, even if the bundled ffmpeg/ffprobe programs are present
// if there's no global installation, we look for the bundled programs that should be in the same directory as icecutter
// if neither are found, the program shows an error message and exits with an error code of 1
fn program_path(program: Program) -> String {
    // convert program to string depending on OS
    let program = match (program, cfg!(windows)) {
        (Program::Ffmpeg, true) => "ffmpeg.exe",
        (Program::Ffmpeg, false) => "ffmpeg",
        
        (Program::Ffprobe, true) => "ffprobe.exe",
        (Program::Ffprobe, false) => "ffprobe"
    };

    // get directory of executable
    let mut exe_dir = env::current_exe().expect("could not get path of the current executable");
    exe_dir.pop();

    // get path of specified program from PATH variable
    // returns Ok(path) if it was found or Err(()) if it wasn't
    let env_path = || -> Result<String, ()> {
        // get PATH environment variable
        let Some(paths) = env::var_os("PATH") else {
            return Err(());
        };

        // iterate over every directory in the PATH
        for path in env::split_paths(&paths) {
            // add "ffmpeg.exe" to the end of the directory path
            let path = path.join(program);

            // check if such path exists, and return it if it does
            // skip the bundled program path, as that counts too as being in the PATH for some reason (on windows anyway) 
            if path.exists() && path != exe_dir.join(program) {
                return Ok(path.to_string_lossy().to_string());
            }
        }

        Err(())
    };

    // get path of specified program in the same directory as icecutter (these are the files that are bundled in with icecutter in the zip file)
    // returns Ok(path) if it was found or Err(()) if it wasn't
    let bundle_path = || -> Result<String, ()> {
        let path = exe_dir.join(program);

        if !path.exists() {
            return Err(())
        }

        Ok(path.to_string_lossy().to_string())
    };

    if let Ok(path) = env_path() {
        println!("using global {program}: {path}");
        return path;
    }

    if let Ok(path) = bundle_path() {
        println!("using bundled {program}: {path}");
        return path;
    }

    error::show_error(format!("failed to find ffmpeg both in the PATH environment variable and in the directory icecutter is in\n\nplease make sure that ffmpeg and ffprobe are in the same directory as icecutter, or install ffmpeg globally instead if you'd like"));
    exit(1);
}

// specifically check if input file is a video file and not an audio file
// this will return "true" for non-audio files like text files and executables and what not
// but other checks will make sure that the input file is a valid video file through fps/length checks
pub fn is_video(video: &str) -> bool {
    let mut command = Command::new(program_path(Program::Ffprobe));
    command
        .arg("-i")
        .arg(video)
        .arg("-show_entries")
        .arg("stream=codec_type")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0");

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = match command.output() {
        Ok(x) => x,
        Err(_) => return false
    };

    let output = String::from_utf8(output.stdout).expect("output has to be valid utf8");
    let output: Vec<&str> = output.split('\n').collect();

    let text = match output.first() {
        Some(x) => x,
        None => return false
    };

    if text.trim() == "video" {
        return true;
    }

    false
}

// returns the ffmpeg version
// this can theoretically panic however by the time this function is called we've already established that we have a valid ffmpeg/ffprobe installation
pub fn program_version(program: Program) -> String {
    let mut command = Command::new(program_path(program));
    command.arg("-version");

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().expect("ffmpeg/ffprobe has to be valid and installed correctly");
    let output = String::from_utf8(output.stdout).expect("output has to be valid utf8");
    let output: Vec<&str> = output.split(' ').collect();

    output[2].to_owned()
}

// returns the length of the video in MM:SS format
pub fn video_length(video: &str) -> Result<String, Box<dyn Error>> {
    // ffprobe command to get video length in HOURS:MM:SS.MICROSECONDS format
    let mut command = Command::new(program_path(Program::Ffprobe));
    command
        .arg("-i")
        .arg(video)
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0")
        .arg("-sexagesimal");
    
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output()?;

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
    let mut command = Command::new(program_path(Program::Ffprobe)); 
    command.arg("-i")
        .arg(video)
        .arg("-select_streams")
        .arg("v")
        .arg("-show_entries")
        .arg("stream=r_frame_rate")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0");

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output()?;

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

// starts the ffmpeg process to convert the video
// expects sanitized input (correct from/to timestamps, valid FPS, etc.)
// this is ran by conversion_subscription when the user asks to convert a video
// the returned child process gets used to track the output
pub fn convert_process(state: &ui::State, output: &str) -> io::Result<Child> {
    // arg vec that we will push arguments into depending on the configuration
    let mut arguments: Vec<&str> = Vec::new();

    // input file
    arguments.push("-i");
    arguments.push(&state.file);

    // display ffmpeg progress in a more computer friendly format
    arguments.push("-progress");
    arguments.push("pipe:1");

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
    let mut command = Command::new(program_path(Program::Ffmpeg));
    command
        .args(&arguments)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    command.spawn()
}

pub fn conversion_subscription() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        // create channel for the application to communicate with the subscription
        let (msg_tx, mut msg_rx) = mpsc::channel(1024);

        // send the sender to the application
        output.send(Message::SubscriptionReady(msg_tx)).await.unwrap();

        loop {
            // await the Start message
            let ConversionInput::Start{state, output_file} = msg_rx.select_next_some().await;

            let child = match convert_process(&state, &output_file) {
                Ok(x) => x,
                Err(err) => { 
                    output.send(Message::SubscriptionError(format!("failed to start ffmpeg process: {err}"))).await.unwrap();
                    continue;
                }
            };

            let Some(stdout) = child.stdout else {
                output.send(Message::SubscriptionError("failed to capture standard output of ffmpeg process".to_owned())).await.unwrap();
                continue;
            };

            let Some(stderr) = child.stderr else {
                output.send(Message::SubscriptionError("failed to capture standard error output of ffmpeg process".to_owned())).await.unwrap();
                continue;
            };

            // setup channel for communicating with stdout/stderr reader threads
            let (mut process_tx, mut process_rx) = mpsc::channel(64);
            let mut error_tx = process_tx.clone();

            // reads stderr and detects if there are any errors
            // if one is found, it gets sent to the process channel as an Err(String)
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines() {
                    // extract line string from Result
                    let Ok(line) = line else {
                        continue;
                    };

                    if line.starts_with("Error") {
                        let _ = error_tx.try_send(Err(line));
                    }
                }
            });
            
            // reads ffmpeg output, and tries to find the 'out_time=' line
            // once its found, it gets sent to the process channel as an Ok(String) (the string being the progress number)
            std::thread::spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    // extract line string from Result
                    let Ok(line) = line else {
                        continue;
                    };

                    // found out_time= line, strip the prefix and send to channel
                    if line.starts_with("out_time=") {
                        let progress = line.strip_prefix("out_time=")
                            .expect("we have already verified that the string contains the prefix")
                            .to_owned();
                        
                        // ignore error, it's ok if the messages don't make it
                        let _ = process_tx.try_send(Ok(progress));
                    }
                }
            });

            // read from progress channel, Ok means to send a subscription progress message, Err means to send a SubscriptionError message
            while let Some(input) = process_rx.next().await {
                match input {
                    Ok(progress) => output.send(Message::SubscriptionProgress(progress)).await.unwrap(),
                    Err(err) => output.send(Message::SubscriptionError(err)).await.unwrap()
                }
            }

            // no more messages from the progress channel, we have completed the operation
            output.send(Message::SubscriptionFinished).await.unwrap();
        }
    })
}