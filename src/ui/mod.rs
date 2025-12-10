use std::path::{Path, PathBuf};
use std::result::Result;

use iced::*;
use iced::widget::{*, column};

pub mod conversion;
mod validation;
mod dialog;

use conversion::{Conversion, ConversionState};
use dialog::Dialog;

use crate::ffmpeg;
use crate::timestamp::Timestamp;

#[derive(Debug, Clone)]
pub struct State {
    pub from: Timestamp,
    pub to: Timestamp,

    pub from_str: String,
    pub to_str: String,

    pub file: PathBuf,
    pub fps: String,

    pub lower_720p: bool,
    pub copy_clipboard: bool,

    pub conversion: ConversionState
}

impl Default for State {
    fn default() -> Self {
        Self {
            from: Timestamp::default(),
            to: Timestamp::default(),

            from_str: String::default(),
            to_str: String::default(),

            file: PathBuf::default(),
            fps: String::default(),

            lower_720p: true,
            copy_clipboard: true,
            
            conversion: ConversionState::default(),
        }
    }
}

impl State {
    // initializes the state from a video file
    pub fn new(file: &Path) -> Result<State, String> {
        let length = ffmpeg::video_length(file)
            .map_err(|err| format!("failed to get video length: {err}"))?;

        let fps = ffmpeg::video_fps(file)
            .map_err(|err| format!("failed to get video fps: {err}"))?;

        if !ffmpeg::is_video(file) {
            return Err("input file does not contain a valid video stream".to_owned());
        }

        let empty_timestamp = Timestamp::new("00:00").unwrap();

        Ok(State {
            from: empty_timestamp.clone(),
            to: length.clone(),

            from_str: empty_timestamp.to_string(),
            to_str: length.to_string(),

            file: file.into(),
            fps,

            ..Default::default()
        })
    }
}


#[derive(Debug, Clone)]
pub enum StateUpdate {
    From(String),
    To(String),

    File(String),
    Fps(String),

    Lower720p(bool),
    CopyClipboard(bool),

    New(State)
}

#[derive(Debug, Clone)]
pub enum Action {
    None,

    StateUpdate(StateUpdate),
    Conversion(Conversion),
    Dialog(Dialog)
}

impl State {
    pub fn update(&mut self, message: Action) -> Task<Action> {
        match message {
            // dummy message
            Action::None => Task::none(),

            // state changes
            Action::StateUpdate(msg) => {
                match msg {
                    StateUpdate::From(s) => {
                        let Ok(timestamp) = Timestamp::new(&s) else {
                            return Task::none() 
                        };

                        self.from = timestamp;
                        self.from_str = s;
                    },
                    StateUpdate::To(s) => {
                        let Ok(timestamp) = Timestamp::new(&s) else {
                            return Task::none() 
                        };

                        self.to = timestamp;
                        self.to_str = s;
                    },
                    
                    StateUpdate::File(s) => self.file = PathBuf::from(s),
                    StateUpdate::Fps(s) => self.fps = s,

                    StateUpdate::Lower720p(b) => self.lower_720p = b,
                    StateUpdate::CopyClipboard(b) => self.copy_clipboard = b,

                    // let's keep our conversion state
                    StateUpdate::New(state) => *self = State {
                        conversion: self.conversion.clone(),
                        ..state
                    }
                };

                Task::none()
            },

            // conversion messages
            Action::Conversion(msg) => conversion::update(&mut self.conversion, msg),

            // file dialog messages
            Action::Dialog(msg) => dialog::update(self, msg)
        }
    }

    // main view
    pub fn view(&self) -> Container<'_, Action> {
        container(column![
            center(
                column![
                    // text column
                    column![
                        text("icecutter v1.3.2")
                            .size(30),
                        text("takes a video file, cuts it, and then compresses it down to 10MB or less with the specified configuration"),
                        text("primarly built for quickly cutting and compressing clips to upload to discord")
                            .size(12)
                    ]
                    .spacing(5),

                    // separator
                    rule::horizontal(1),

                    // from:to textboxes
                    row![
                        text_input("from", &self.from_str)
                            .on_input(|s| Action::StateUpdate(StateUpdate::From(s))),
                        text("-")
                            .size(20),
                        text_input("to", &self.to_str)
                            .on_input(|s| Action::StateUpdate(StateUpdate::To(s))),
                    ]
                    .spacing(10)
                    .width(150),
                    
                    // input file
                    row![
                        button("select")
                            .on_press(Action::Dialog(Dialog::Select)),

                        space()
                            .width(10),
                            
                        container(
                            text_input("input file", &self.file.to_string_lossy())
                                .on_input(|s| Action::StateUpdate(StateUpdate::File(s))),
                        )
                        .width(300),
                    ],
                    
                    // fps
                    container(
                        text_input("fps", &self.fps)
                            .on_input(|s| Action::StateUpdate(StateUpdate::Fps(s))),
                    )
                    .width(75),
                    
                    // 720p checkbox
                    checkbox(self.lower_720p)
                        .label("lower resolution to 720p")
                        .on_toggle(|b| Action::StateUpdate(StateUpdate::Lower720p(b))),

                    // copy to clipboard checkbox
                    checkbox(self.copy_clipboard)
                        .label("copy to clipboard")
                        .on_toggle(|b| Action::StateUpdate(StateUpdate::CopyClipboard(b))),
                    
                    // errors
                    validation::error_view(self),

                    // convert button
                    button("convert")
                        .on_press_maybe(match (validation::validate_state(self).is_empty(), self.conversion.converting) {
                            (true, false) => Some(Action::Dialog(Dialog::Convert)),
                            _ => None
                        }),

                    // progress bar
                    conversion::progress_view(&self.conversion)
                ]
                .align_x(Center)
                .padding(20)
                .spacing(8)
            ),

            // ffmpeg version at the bottom left
            row![
                space()
                    .width(5),

                container(
                    text!("ffmpeg version: {}", ffmpeg::program_version(ffmpeg::Program::Ffmpeg))
                        .size(11)
                ).padding(2),
            ],
            
            space()
                .height(2)
        ])
    }

    pub fn subscription(&self) -> Subscription<Action> {
        Subscription::run(ffmpeg::conversion_subscription)
    }
}