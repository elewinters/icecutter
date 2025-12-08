use iced::Task;
use iced::*;
use iced::widget::{*, column};

use std::path::PathBuf;
use iced::futures::channel::mpsc;
use iced::futures::SinkExt;

use crate::ui::{Message, State};
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
    fn start(&mut self, from: String, to: String) {
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

impl ConversionState {
    pub fn update(&mut self, message: ConversionMessage) -> Task<Message> {
        match message {
            // conversion subscription messages
            ConversionMessage::Ready(sender) => {
                self.channel = Some(sender);
                Task::none()
            }

            ConversionMessage::Begin(state, output_file) => {
                let mut sender = self.channel.clone().expect("channel has already been in initialized with the ConversionReady message");

                self.start(state.from.clone(), state.to.clone());
                Task::perform(async move { sender.send(ConversionInput::Start{state, output_file}).await }, |_| Message::None)
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

                self.progress = crate::timestamp_to_secs(&progress);

                Task::none()
            }

            ConversionMessage::Finished(path) => {
                let mut clipboard = Clipboard::new().unwrap();

                clipboard.set().file_list(&[path]).unwrap();
                
                self.reset();
                Task::none()
            }
        }
    }

    pub fn progress_view(&self) -> Element<'_, Message> {
        if !self.converting {
            return space().into();
        }

        let max = crate::timestamp_to_secs(&self.to) - crate::timestamp_to_secs(&self.from);
        let percentage = (self.progress / max * 100.0).floor();

        column![
            rule::horizontal(1),
            text!("processing with ffmpeg: {percentage}%"),
            progress_bar(0.0..=max, self.progress)
                .girth(15)
        ]
        .align_x(Center)
        .padding(5)
        .spacing(10)
        .into()
    }
}