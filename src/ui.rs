use std::path::Path;
use std::result::Result;

use iced::*;
use iced::widget::{*, column};

use iced::futures::channel::mpsc;
use iced::futures::SinkExt;

use arboard::Clipboard;

use crate::error_async;
use crate::ffmpeg::{self, ConversionInput};

#[derive(Default, Clone)]
pub struct ConversionState {
    pub converting: bool,
    pub progress: f32,

    // for communicating with the conversion subscription
    pub channel: Option<mpsc::Sender<ConversionInput>>,

    // we make a copy so that the user can still freely change the timestamps while the conversion is happening
    pub from: String,
    pub to: String
}

#[derive(Clone)]
pub struct State {
    pub from: String,
    pub to: String,

    pub file: String,
    pub fps: String,

    pub convert_720p: bool,
    pub clipboard: bool,

    pub conversion: ConversionState
}

impl Default for State {
    fn default() -> Self {
        Self {
            from: String::default(),
            to: String::default(),

            file: String::default(),
            fps: String::default(),

            convert_720p: true,
            clipboard: true,
            
            conversion: ConversionState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    None,

    ChangeFrom(String),
    ChangeTo(String),
    ChangeFile(String),
    ChangeFps(String),
    ChangeConvert720p(bool),
    ChangeClipboard(bool),

    SubscriptionReady(mpsc::Sender<ConversionInput>),
    SubscriptionError(String),
    SubscriptionProgress(String),
    SubscriptionFinished(String),

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

    // if minutes and seconds exist and if they're both valid u32 integers that arent greater than 59, we return true
    match (minutes, seconds) {
        (Some(x), Some(y)) => {
            let Ok(minutes) = x.parse::<u32>() else {
                return false
            };

            let Ok(seconds) = y.parse::<u32>() else {
                return false
            };

            if minutes > 59 || seconds > 59 {
                return false
            }

            true
        }
        _ => false
    }
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

    // check if "to" timestamp isn't 00:00
    if state.to == "00:00" || state.to == "0:00" || state.to == "00:0" || state.to == "0:0" {
        errors.push("'to' timestamp can't be 00:00".to_owned());
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
    // set all the necessary fields when a conversion begins
    fn start_conversion_state(&mut self) {
        self.conversion.converting = true;
        self.conversion.from = self.from.clone();
        self.conversion.to = self.to.clone();
    }

    // reset the state after a conversion has finished
    fn reset_conversion_state(&mut self) {
        self.conversion.converting = false;
        self.conversion.progress = 0.0;
    }

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
            Message::ChangeClipboard(b) => {
                self.clipboard = b;
                Task::none()
            }

            // conversion subscription messages
            Message::SubscriptionReady(sender) => {
                self.conversion.channel = Some(sender);
                Task::none()
            }

            Message::SubscriptionError(err) => error_async!("ffmpeg error: {err}"),

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
            Message::SubscriptionFinished(path) => {
                if self.clipboard {
                    let mut clipboard = Clipboard::new().unwrap();

                    let paths = [Path::new(&path)];
                    clipboard.set().file_list(&paths).unwrap();
                }
                
                self.reset_conversion_state();
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
                // get filehandle if a file was successfully picked
                let Some(file) = file_opt else {
                    return Task::none()
                };

                // get the selected file path
                let path = file.path().to_string_lossy();

                // initialize state based on selected file
                let state = match initialize_state(&path) {
                    Ok(x) => x,
                    Err(err) => return error_async!("failed to initialize state: {err}")
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
                let length = match initialize_state(&path.to_string_lossy()) {
                    Ok(state) => state.to,
                    Err(err) => return error_async!("invalid input file: {err}")
                };

                // additionally check if to timestamp is bigger than the video's length
                if crate::timestamp_to_secs(&self.to) > crate::timestamp_to_secs(&length) {
                    return error_async!("'to' timestamp is longer than the video's duration");
                }

                // and check if "from" is bigger than "to"
                if crate::timestamp_to_secs(&self.from) > crate::timestamp_to_secs(&self.to) {
                    return error_async!("'from' timestamp is longer than the 'to' timestamp");
                }

                // and check if the timestamps are the same
                if crate::timestamp_to_secs(&self.from) == crate::timestamp_to_secs(&self.to) {
                    return error_async!("the 'from' and 'to' timestamps cannot be the same");
                }

                let file_name = match path.file_name() {
                    Some(x) => format!("[converted] {}", x.to_string_lossy()),
                    None => return error_async!("invalid input file, failed to get file name from path")
                };

                Task::perform(
                    rfd::AsyncFileDialog::new()
                        .set_title("save converted video")
                        .set_file_name(file_name)
                        .save_file(),
                    Message::ConvertDialogFinished // once the file dialog task is over, run this message
                )
            }

            // ran when the convert file dialog has finished
            Message::ConvertDialogFinished(file_opt) => {
                // get filehandle if a file was successfully picked
                let Some(file) = file_opt else {
                    return Task::none(); 
                };

                // get the selected file path
                let output_file = file.path().to_string_lossy().to_string();
                
                // send a message to the subscription to start the conversion with ffmpeg
                let mut sender = self.conversion.channel.clone().expect("channel has already been in initialized with the SubscriptionReady message");
                let state = self.clone();

                self.start_conversion_state();
                
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

        let max = crate::timestamp_to_secs(&self.conversion.to) - crate::timestamp_to_secs(&self.conversion.from);
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
                        text("icecutter v1.3.2")
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

                    // copy to clipboard checkbox
                    checkbox("copy to clipboard", self.clipboard)
                        .on_toggle(Message::ChangeClipboard),
                    
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
                .padding(20)
                .spacing(8)
            ),

            // ffmpeg version at the bottom left
            row![
                Space::new(10, 0),
                container(
                    text("ffmpeg version: ".to_owned() + &ffmpeg::program_version(ffmpeg::Program::Ffmpeg))
                        .size(11)
                ).padding(2),
            ],

        ])
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(ffmpeg::conversion_subscription)
    }
}