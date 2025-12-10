use std::path::PathBuf;

use iced::{Task, Center, Element};
use iced::widget::{*, column};

use iced::futures::channel::mpsc;
use iced::futures::SinkExt;

use arboard::Clipboard;

use crate::timestamp::Timestamp;
use crate::ui::{Action, State};
use crate::error_async;
use crate::ffmpeg::ConversionInput;

#[derive(Debug, Clone)]
pub enum Conversion {
    Ready(mpsc::Sender<ConversionInput>),
    
    Start(State, PathBuf),

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
    pub from: Timestamp,
    pub to: Timestamp,

    pub copy_clipboard: bool
}

impl ConversionState {
    // set all the necessary fields when a conversion begins
    fn set(&mut self, copy_clipboard: bool, from: Timestamp, to: Timestamp) {
        self.converting = true;
        self.copy_clipboard = copy_clipboard;

        self.from = from;
        self.to = to;
    }

    // reset the state after a conversion has finished
    // we reset everything except for the channel we use to communicate with the conversion subscription
    fn reset(&mut self) {
        *self = ConversionState {
            channel: self.channel.clone(),
            ..Default::default()
        };
    }
}

pub fn update(conversion: &mut ConversionState, message: Conversion) -> Task<Action> {
    match message {
        // conversion subscription messages
        Conversion::Ready(sender) => {
            conversion.channel = Some(sender);
            Task::none()
        }

        Conversion::Start(state, output_file) => {
            let mut sender = conversion.channel.clone().expect("channel has already been in initialized with the ConversionReady message");

            conversion.set(state.copy_clipboard, state.from.clone(), state.to.clone());
            Task::perform(async move { sender.send(ConversionInput::Start{state, output_file}).await }, |_| Action::None)
        }

        Conversion::Error(err) => error_async!("ffmpeg conversion error: {err}"),

        Conversion::Progress(progress) => {
            if progress == "N/A" {
                return Task::none();
            }

            conversion.progress = Timestamp::new(&progress).unwrap_or_default().total_secs();
            Task::none()
        }

        Conversion::Finished(path) => {
            if conversion.copy_clipboard {
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set().file_list(&[path]).unwrap();
            }
            
            conversion.reset();
            Task::none()
        }
    }
}

pub fn progress_view(conversion: &ConversionState) -> Element<'_, Action> {
    if !conversion.converting {
        return space().into();
    }

    let max = conversion.to.total_secs() - conversion.from.total_secs();
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