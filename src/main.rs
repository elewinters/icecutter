use std::env;

mod ui;
mod error;
mod ffmpeg;

fn main() {
    // get input video file command line argument, this can be None
    let file: Option<String> = env::args().nth(1);

    // get length and FPS of video if the file argument exists
    let length: Option<String> = file.as_ref().map(|f| ffmpeg::video_length(f)).transpose().unwrap_or_else(|err| {
        error::show_error(format!("failed to get video length: {err}"));
        None
    });

    let fps: Option<String> = file.as_ref().map(|f| ffmpeg::video_fps(f)).transpose().unwrap_or_else(|err| {
        error::show_error(format!("failed to get video fps: {err}"));
        None 
    });

    // run iced application with custom initial state
    let result = iced::application("icecutter", ui::State::update, ui::State::view)
        .window(iced::window::Settings {
            size: (640.0, 480.0).into(),
            resizable: false,
            ..Default::default()
        })
        .run_with(|| {(
            ui::State {
                // set "from" string to 00:00 if length is valid
                from: length.as_ref().map_or(String::default(), |_| String::from("00:00")),
                to: length.unwrap_or_default(),

                file: file.unwrap_or_default(),
                fps: fps.unwrap_or_default(),

                ffmpeg_installed: ffmpeg::check_program("ffmpeg").is_ok(),
                ffprobe_installed: ffmpeg::check_program("ffprobe").is_ok(),

                ..Default::default()
            },
            iced::Task::none()
        )});
    
    if let Err(err) = result {
        error::show_error(format!("failed to initialize iced, something must've went very wrong: {err}"));
    }
}