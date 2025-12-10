use iced::Task;
use crate::error_async;

use super::State;
use super::conversion::Conversion;

use crate::ui::{Action, StateUpdate};

#[derive(Debug, Clone)]
pub enum Dialog {
    Select,
    SelectFinished(Option<rfd::FileHandle>),

    Convert,
    ConvertFinished(Option<rfd::FileHandle>)
}

pub fn update(state: &State, message: Dialog) -> Task<Action<>> {
    match message {
        Dialog::Select => Task::perform(
            rfd::AsyncFileDialog::new()
                .set_title("select video to convert")
                .pick_file(),
            |x| Action::Dialog(Dialog::SelectFinished(x)) // once the file dialog task is over, run this message
        ),

        Dialog::SelectFinished(file_opt) => {
            // get filehandle if a file was successfully picked
            let Some(file) = file_opt else {
                return Task::none()
            };

            // initialize state based on selected file
            let state = match State::new(file.path()) {
                Ok(x) => x,
                Err(err) => return error_async!("failed to initialize state: {err}")
            };

            Task::done(Action::StateUpdate(StateUpdate::New(state)))
        }

        Dialog::Convert => {
            // check if input file is a valid video
            // yes this may result in State::new being called twice if the user has used the select file dialog, however the user can also input the file path without using it
            // in which case if the user inputted a non-video into that field, ffmpeg would error out
            let length = match State::new(&state.file) {
                Ok(state) => state.to,
                Err(err) => return error_async!("invalid input file: {err}")
            };

            // additionally check if to timestamp is bigger than the video's length
            if state.to.total_secs() > length.total_secs() {
                return error_async!("'to' timestamp is longer than the video's duration");
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
                |x| Action::Dialog(Dialog::ConvertFinished(x))  // once the file dialog task is over, run this message
            )
        }

        Dialog::ConvertFinished(file_opt) => {
            // get filehandle if a file was successfully picked
            let Some(file) = file_opt else {
                return Task::none(); 
            };

            // send a message to the subscription to start the conversion with ffmpeg
            let output_file = file.path().to_path_buf();
            let state = state.clone();

            Task::done(Action::Conversion(Conversion::Begin(state, output_file)))
        }
    }
}