use iced::Task;
use iced::*;
use iced::widget::{*, column};

use std::path::PathBuf;
use iced::futures::channel::mpsc;
use iced::futures::SinkExt;

use crate::ui::{Action, State};
use arboard::Clipboard;

use crate::error_async;
use crate::ffmpeg::ConversionInput;

#[derive(Debug, Clone)]
pub enum ConversionMessage {
    Ready(mpsc::Sender<ConversionInput>),
    Begin(State, PathBuf),
    Error(String),
    Progress(String),
    Finished(PathBuf),
}

#[derive(Default, Debug, Clone)]
pub struct ConversionState {
    pub converting: bool,
    pub progress: f32,

    // for communicating with the conversion subscription
    pub channel: Option<mpsc::Sender<ConversionInput>>,

    // we make a copy so that the user can still freely change the timestamps while the conversion is happening
    pub from: String,
    pub to: String
}

impl ConversionState {
    // set all the necessary fields when a conversion begins
    fn set(&mut self, from: String, to: String) {
        self.converting = true;
        self.from = from;
        self.to = to;
    }

    // reset the state after a conversion has finished
    fn reset(&mut self) {
        self.converting = false;
        self.progress = 0.0;
    }
}

pub fn update(conversion: &mut ConversionState, message: ConversionMessage) -> Task<Action> {
    match message {
        // conversion subscription messages
        ConversionMessage::Ready(sender) => {
            conversion.channel = Some(sender);
            Task::none()
        }

        ConversionMessage::Begin(state, output_file) => {
            let mut sender = conversion.channel.clone().expect("channel has already been in initialized with the ConversionReady message");

            conversion.set(state.from.clone(), state.to.clone());
            Task::perform(async move { sender.send(ConversionInput::Start{state, output_file}).await }, |_| Action::None)
        }

        ConversionMessage::Error(err) => error_async!("ffmpeg conversion error: {err}"),

        ConversionMessage::Progress(mut progress) => {
            if progress == "N/A" {
                return Task::none();
            }
            
            // remove hour
            progress.remove(0);
            progress.remove(0);
            progress.remove(0);

            conversion.progress = crate::timestamp_to_secs(&progress);

            Task::none()
        }

        ConversionMessage::Finished(path) => {
            let mut clipboard = Clipboard::new().unwrap();

            clipboard.set().file_list(&[path]).unwrap();
            
            conversion.reset();
            Task::none()
        }
    }
}

pub fn progress_view(conversion: &ConversionState) -> Element<'_, Action> {
    if !conversion.converting {
        return space().into();
    }

    let max = crate::timestamp_to_secs(&conversion.to) - crate::timestamp_to_secs(&conversion.from);
    let percentage = (conversion.progress / max * 100.0).floor();

    column![
        rule::horizontal(1),
        text!("processing with ffmpeg: {percentage}%"),
        progress_bar(0.0..=max, conversion.progress)
            .girth(15)
    ]
    .align_x(Center)
    .padding(5)
    .spacing(10)
    .into()
}