use iced::Task;
use crate::error_async;

use super::conversion::*;
use super::State;

use crate::ui::{Action, StateMessage, initialize_state};

#[derive(Debug, Clone)]
pub enum DialogMessage {
    Select,
    SelectFinished(Option<rfd::FileHandle>),

    Convert,
    ConvertFinished(Option<rfd::FileHandle>)
}

pub fn update(state: &State, message: DialogMessage) -> Task<Action<>> {
    match message {
        DialogMessage::Select => Task::perform(
            rfd::AsyncFileDialog::new()
                .set_title("select video to convert")
                .pick_file(),
            |x| Action::Dialog(DialogMessage::SelectFinished(x)) // once the file dialog task is over, run this message
        ),

        DialogMessage::SelectFinished(file_opt) => {
            // get filehandle if a file was successfully picked
            let Some(file) = file_opt else {
                return Task::none()
            };

            // initialize state based on selected file
            let state = match initialize_state(file.path()) {
                Ok(x) => x,
                Err(err) => return error_async!("failed to initialize state: {err}")
            };

            Task::done(Action::UpdateState(StateMessage::NewState(state)))
        }

        DialogMessage::Convert => {
            // check if input file is a valid video
            // yes this may result in initialize_state being called twice if the user has used the select file dialog, however the user can also input the file path without using it
            // in which case if the user inputted a non-video into that field, ffmpeg would error out
            let length = match initialize_state(&state.file) {
                Ok(state) => state.to,
                Err(err) => return error_async!("invalid input file: {err}")
            };

            // additionally check if to timestamp is bigger than the video's length
            if crate::timestamp_to_secs(&state.to) > crate::timestamp_to_secs(&length) {
                return error_async!("'to' timestamp is longer than the video's duration");
            }

            // and check if "from" is bigger than "to"
            if crate::timestamp_to_secs(&state.from) > crate::timestamp_to_secs(&state.to) {
                return error_async!("'from' timestamp is longer than the 'to' timestamp");
            }

            // and check if the timestamps are the same
            if crate::timestamp_to_secs(&state.from) == crate::timestamp_to_secs(&state.to) {
                return error_async!("the 'from' and 'to' timestamps cannot be the same");
            }

            let file_name = match state.file.file_name() {
                Some(x) => format!("[converted] {}", x.to_string_lossy()),
                None => return error_async!("invalid input file, failed to get file name from path")
            };

            Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("save converted video")
                    .set_file_name(file_name)
                    .save_file(),
                |x| Action::Dialog(DialogMessage::ConvertFinished(x))  // once the file dialog task is over, run this message
            )
        }

        DialogMessage::ConvertFinished(file_opt) => {
            // get filehandle if a file was successfully picked
            let Some(file) = file_opt else {
                return Task::none(); 
            };

            // send a message to the subscription to start the conversion with ffmpeg
            let output_file = file.path().to_path_buf();
            let state = state.clone();

            Task::done(Action::Conversion(ConversionMessage::Begin(state, output_file)))
        }
    }
}