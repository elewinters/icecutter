use std::path::{Path, PathBuf};
use std::result::Result;

use iced::*;
use iced::widget::{*, column};

use crate::ffmpeg;

pub mod conversion;
use conversion::*;

mod dialog;
use dialog::DialogMessage;

mod validation;

#[derive(Debug, Clone)]
pub struct State {
    pub from: String,
    pub to: String,

    pub file: PathBuf,
    pub fps: String,

    pub lower_720p: bool,
    pub clipboard: bool,

    pub conversion: ConversionState
}

impl Default for State {
    fn default() -> Self {
        Self {
            from: String::default(),
            to: String::default(),

            file: PathBuf::default(),
            fps: String::default(),

            lower_720p: true,
            clipboard: true,
            
            conversion: ConversionState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateMessage {
    From(String),
    To(String),

    File(String),
    Fps(String),

    Lower720p(bool),
    CopyClipboard(bool),

    NewState(State)
}

#[derive(Debug, Clone)]
pub enum Action {
    None,

    UpdateState(StateMessage),
    Conversion(ConversionMessage),
    Dialog(DialogMessage)
}

// initializes the state from a video file
pub fn initialize_state(file: &Path) -> Result<State, String> {
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

        file: file.into(),
        fps,

        ..Default::default()
    })
}

impl State {
    pub fn update(&mut self, message: Action) -> Task<Action> {
        match message {
            // dummy message
            Action::None => Task::none(),

            // state changes
            Action::UpdateState(msg) => {
                match msg {
                    StateMessage::From(s) => self.from = s,
                    StateMessage::To(s) => self.to = s,
                    
                    StateMessage::File(s) => self.file = PathBuf::from(s),
                    StateMessage::Fps(s) => self.fps = s,

                    StateMessage::Lower720p(b) => self.lower_720p = b,
                    StateMessage::CopyClipboard(b) => self.clipboard = b,

                    // let's keep our conversion state
                    StateMessage::NewState(state) => *self = State {
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
                        text_input("from", &self.from)
                            .on_input(|s| Action::UpdateState(StateMessage::From(s))),
                        text("-")
                            .size(20),
                        text_input("to", &self.to)
                            .on_input(|s| Action::UpdateState(StateMessage::To(s))),
                    ]
                    .spacing(10)
                    .width(150),
                    
                    // input file
                    row![
                        button("select")
                            .on_press(Action::Dialog(DialogMessage::Select)),

                        space()
                            .width(10),
                            
                        container(
                            text_input("input file", &self.file.to_string_lossy())
                                .on_input(|s| Action::UpdateState(StateMessage::File(s))),
                        )
                        .width(300),
                    ],
                    
                    // fps
                    container(
                        text_input("fps", &self.fps)
                            .on_input(|s| Action::UpdateState(StateMessage::Fps(s))),
                    )
                    .width(75),
                    
                    // 720p checkbox
                    checkbox(self.lower_720p)
                        .label("lower resolution to 720p")
                        .on_toggle(|b| Action::UpdateState(StateMessage::Lower720p(b))),

                    // copy to clipboard checkbox
                    checkbox(self.clipboard)
                        .label("copy to clipboard")
                        .on_toggle(|b| Action::UpdateState(StateMessage::CopyClipboard(b))),
                    
                    // errors
                    validation::error_view(self),

                    // convert button
                    button("convert")
                        .on_press_maybe(match (validation::validate_state(self).is_empty(), self.conversion.converting) {
                            (true, false) => Some(Action::Dialog(DialogMessage::Convert)),
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