use std::path::Path;
use std::result::Result;

use iced::futures::channel::mpsc;
use iced::futures::SinkExt;

use iced::*;
use iced::widget::{*, column};

use crate::error;
use crate::ffmpeg;

use ffmpeg::ConversionInput;

#[derive(Default, Clone)]
pub struct ConversionState {
    pub converting: bool,
    pub progress: f32,
    pub channel: Option<mpsc::Sender<ConversionInput>>
}

#[derive(Default, Clone)]
pub struct State {
    pub from: String,
    pub to: String,

    pub file: String,
    pub fps: String,
    pub convert_720p: bool,

    pub conversion: ConversionState
}

#[derive(Debug, Clone)]
pub enum Message {
    None,

    ChangeFrom(String),
    ChangeTo(String),
    ChangeFile(String),
    ChangeFps(String),
    ChangeConvert720p(bool),

    SubscriptionReady(mpsc::Sender<ConversionInput>),
    SubscriptionProgress(String),
    SubscriptionFinished,

    SelectDialog,
    SelectDialogFinished(Option<rfd::FileHandle>),

    ConvertDialog,
    ConvertDialogFinished(Option<rfd::FileHandle>),
}

// initializes the state from a video file
pub fn initialize_state(file: &str) -> Result<State, String> {
    if !Path::new(file).exists() {
        return Err(format!("file \"{file}\" does not exist"))
    }

    let length = ffmpeg::video_length(file)
        .map_err(|err| format!("failed to get video length: {err}"))?;

    let fps = ffmpeg::video_fps(file)
        .map_err(|err| format!("failed to get video fps: {err}"))?;

    if !ffmpeg::is_video(file) {
        return Err("input file does not contain a valid video stream".to_owned());
    }

    Ok(State {
        from: "00:00".to_owned(),
        to: length,

        file: file.to_string(),
        fps,

        ..Default::default()
    })
}

// checks if the from/to timestamps are valid
fn validate_timestamp(timestamp: &str) -> bool {
    // an empty timestamp is also valid
    if timestamp.is_empty() {
        return true;
    }

    let split: Vec<&str> = timestamp.split(':').collect();

    let minutes = split.first();
    let seconds = split.get(1);

    // return false if we have more than one colon
    if split.len() > 2 {
        return false
    }

    // if minutes and seconds exist and if they're both valid u32 integers, we return true
    matches!((minutes, seconds), (Some(x), Some(y)) if x.parse::<u32>().is_ok() && y.parse::<u32>().is_ok())
}

// checks if all of the input values are valid, and returns a vector of string errors
// if the state is valid, then an empty vector is returned
fn validate_state(state: &State) -> Vec<String> {
    let mut errors = Vec::new();

    // check if only one of the timestamps is filled and throw an error
    if state.to.is_empty() && !state.from.is_empty() {
        errors.push("'to' timestamp is empty, while the 'from' timestamp is not".to_owned());
    }

    if state.from.is_empty() && !state.to.is_empty() {
        errors.push("'from' timestamp is empty, while the 'to' timestamp is not".to_owned());
    }

    // check if both timestamps are in the proper format
    // if one of these is empty it will also return true
    if !validate_timestamp(&state.from) {
        errors.push("'from' timestamp is not in a valid MM:SS format".to_owned());
    }

    if !validate_timestamp(&state.to) {
        errors.push("'to' timestamp is not in a valid MM:SS format".to_owned());
    }

    // check if input file field is empty
    if state.file.is_empty() {
        errors.push("'input file' field is empty".to_owned());
    }

    // check if fps field is a valid number
    if state.fps.parse::<u32>().is_err() {
        errors.push("'fps' field is not a valid unsigned integer".to_owned())
    }

    // check if file exists
    if !state.file.is_empty() && !Path::new(&state.file).exists() {
        errors.push("input file does not exist".to_owned());
    }

    errors
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // dummy message
            Message::None => Task::none(),

            // state changes
            Message::ChangeFrom(from) => {
                self.from = from;
                Task::none()
            }
            Message::ChangeTo(to) => {
                self.to = to;
                Task::none()
            }
            Message::ChangeFile(file) => {
                self.file = file;
                Task::none()
            }
            Message::ChangeFps(fps) => {
                self.fps = fps;
                Task::none()
            }
            Message::ChangeConvert720p(convert) => {
                self.convert_720p = convert;
                Task::none()
            }

            // conversion subscription messages
            Message::SubscriptionReady(sender) => {
                self.conversion.channel = Some(sender);
                Task::none()
            }
            Message::SubscriptionProgress(mut progress) => {
                if progress == "N/A" {
                    return Task::none();
                }
                
                // remove hour
                progress.remove(0);
                progress.remove(0);
                progress.remove(0);

                self.conversion.progress = crate::timestamp_to_secs(&progress);

                Task::none()
            }
            Message::SubscriptionFinished => {
                self.conversion.converting = false;
                self.conversion.progress = 0.0;
                Task::none()
            }

            // ran upon clicking the select button
            Message::SelectDialog => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("select video to convert")
                    .pick_file(),
                Message::SelectDialogFinished // once the file dialog task is over, run this message
            ),

            // ran when the select file dialog has finished
            Message::SelectDialogFinished(file_opt) => {
                // verify if file dialog succeeded
                let Some(file) = file_opt else {
                    return Task::none()
                };

                // get the selected file path
                let path = file.path().to_string_lossy();

                // initialize state based on selected file
                let state = match initialize_state(&path) {
                    Ok(x) => x,
                    Err(err) => return Task::perform(error::show_error_async(format!("failed to initialize state: {err}")), |_| Message::None)
                };

                *self = State {
                    conversion: self.conversion.clone(),
                    ..state
                };
                Task::none()
            }

            // ran upon clicking the convert button
            Message::ConvertDialog => {
                let path = Path::new(&self.file);

                // check if input file is a valid video
                // yes this may result in initialize_state being called twice if the user has used the select file dialog, however the user can also input the file path without using it
                // in which case if the user inputted a non-video into that field, ffmpeg would error out
                if let Err(err) = initialize_state(&path.to_string_lossy()) {
                    return Task::perform(error::show_error_async(format!("invalid input file: {err}")), |_| Message::None);
                }

                // get the directory of the file
                let directory = match path.parent() {
                    Some(path) => path.to_path_buf(),
                    None => return Task::perform(error::show_error_async("invalid input file, failed to get parent directory from path"), |_| Message::None)
                };

                // get the file name of the file
                let file_name = match path.file_name() {
                    Some(x) => format!("[converted] {}", x.to_string_lossy()),
                    None => return Task::perform(error::show_error_async("invalid input file, failed to get file name from path"), |_| Message::None)
                };

                Task::perform(
                    rfd::AsyncFileDialog::new()
                        .set_title("save converted video")
                        .set_file_name(file_name)
                        .set_directory(directory.as_path())
                        .save_file(),
                    Message::ConvertDialogFinished // once the file dialog task is over, run this message
                )
            }

            // ran when the convert file dialog has finished
            Message::ConvertDialogFinished(file_opt) => {
                // get filehandle if valid
                // cancelling the file dialog isnt exactly an error so show_error isnt called
                let Some(file) = file_opt else {
                    return Task::none(); 
                };

                // get the selected file path
                let output_file = file.path().to_string_lossy().to_string();
                
                let mut sender = self.conversion.channel.clone().expect("channel has already been in initialized with the SubscriptionReady message");
                let state = self.clone();

                self.conversion.converting = true;
                Task::perform(async move { sender.send(ConversionInput::Start{state, output_file}).await }, |_| Message::None)
            }
        }
    }

    // view responsible for showing error messages above the convert button
    fn error_view(&self) -> Element<'_, Message> {
        let errors = validate_state(self);
        
        if errors.is_empty() {
            return Space::new(0, 0).into();
        }

        column(
            errors.into_iter()
                .map(|error| {
                    text(error)
                        .color(Color::from_rgb(1.0, 0.0, 0.0))
                        .size(12)
                        .into()
                })
                .collect::<Vec<Element<Message>>>()
        )
        .spacing(5)
        .into()
    }

    fn progress_view(&self) -> Element<'_, Message> {
        if !self.conversion.converting {
            return Space::new(0, 0).into();
        }

        let max = crate::timestamp_to_secs(&self.to) - crate::timestamp_to_secs(&self.from);
        let percentage = (self.conversion.progress / max * 100.0).floor();

        column![
            horizontal_rule(1),
            text(format!("processing with ffmpeg: {percentage}%")),
            progress_bar(0.0..=max, self.conversion.progress)
                .height(15)
        ]
        .align_x(Center)
        .padding(5)
        .spacing(10)
        .into()
    }

    // main view
    pub fn view(&self) -> Container<'_, Message> {
        container(column![
            center(
                column![
                    // text column
                    column![
                        text("icecutter v1.3.0")
                            .size(30),
                        text("takes a video file, cuts it, and then compresses it down to 10MB or less with the specified configuration")
                            .center(),
                        text("primarly built for quickly cutting and compressing clips to upload to discord")
                            .size(12)
                    ]
                    .spacing(5),

                    // separator
                    horizontal_rule(1),

                    // from:to textboxes
                    row![
                        text_input("from", &self.from)
                            .on_input(Message::ChangeFrom),
                        text("-")
                            .size(20),
                        text_input("to", &self.to)
                            .on_input(Message::ChangeTo),
                    ]
                    .spacing(10)
                    .width(150),
                    
                    // input file
                    row![
                        button("select")
                            .on_press(Message::SelectDialog),
                        Space::new(10, 0),
                        container(
                            text_input("input file", &self.file)
                                .on_input(Message::ChangeFile),
                        )
                        .width(300),
                    ],
                    
                    // fps
                    container(
                        text_input("fps", &self.fps)
                            .on_input(Message::ChangeFps),
                    )
                    .width(75),
                    
                    // 720p checkbox
                    checkbox("convert to 720p", self.convert_720p)
                        .on_toggle(Message::ChangeConvert720p),

                    // errors
                    self.error_view(),

                    // convert button
                    button("convert")
                        .on_press_maybe(match (validate_state(self).is_empty(), self.conversion.converting) {
                            (true, false) => Some(Message::ConvertDialog),
                            _ => None
                        }),

                    // progress bar
                    self.progress_view()
                ]
                .align_x(Center)
                .padding(25)
                .spacing(10)
            ),

            // ffmpeg version at the bottom left
            row![
                Space::new(10, 0),
                container(
                    text("ffmpeg version: ".to_owned() + &ffmpeg::program_version(ffmpeg::Program::Ffmpeg))
                        .size(11)
                ).padding(2),
            ],

            Space::new(0, 5)
        ])
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(ffmpeg::conversion_subscription)
    }
}