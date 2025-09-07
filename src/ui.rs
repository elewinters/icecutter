use iced::*;
use iced::widget::{*, column};

#[derive(Default)]
pub struct State {
    pub from: String,
    pub to: String,

    pub file: String,
    pub fps: String,
    pub convert_720p: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFrom(String),
    ChangeTo(String),

    ChangeFile(String),
    ChangeFps(String),
    ChangeConvert720p(bool),

    FileDialog,
    FileSelected(Option<rfd::FileHandle>),
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
        errors.push(String::from("'to' timestamp is empty, while the 'from' timestamp is not"));
    }

    if state.from.is_empty() && !state.to.is_empty() {
        errors.push(String::from("'from' timestamp is empty, while the 'to' timestamp is not"));
    }

    // check if both timestamps are in the proper format
    // if one of these is empty it will also return true
    if !validate_timestamp(&state.from) {
        errors.push(String::from("'from' timestamp is not in a valid MM:SS format"));
    }

    if !validate_timestamp(&state.to) {
        errors.push(String::from("'to' timestamp is not in a valid MM:SS format"));
    }

    // check if input file field is empty
    if state.file.is_empty() {
        errors.push(String::from("'input file' field is empty"));
    }

    // check if fps field is a valid number
    if state.fps.parse::<u32>().is_err() {
        errors.push(String::from("'fps' field is not a valid unsigned integer"))
    }

    errors
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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

            // ran upon clicking the convert button
            Message::FileDialog => {
                let file_name = format!("[converted] {}", &self.file);
                Task::perform(async {
                    rfd::AsyncFileDialog::new()
                        .set_title("save converted video")
                        .set_file_name(file_name)
                        .save_file()
                        .await
                    },
                    Message::FileSelected // once the file dialog task is over, run this message
                )
            }

            // the file dialog task has finished, so now we run this
            Message::FileSelected(file) => {
                // get filehandle if valid
                let Some(output) = file else {
                    return Task::none() 
                };

                // convert filehandle to string, if possible
                let Some(path) = output.path().to_str() else { 
                    return Task::none() 
                };
                
                // convert the input file with the specified state
                // ignores errors 
                if super::convert(self, path).is_err() {
                    return Task::none()
                }
                
                Task::none()
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

    // main view
    pub fn view(&self) -> Container<'_, Message> {
        center(
            column![
                // text column
                column![
                    text("icecutter")
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
                container(
                    text_input("input file", &self.file)
                        .on_input(Message::ChangeFile),
                )
                .width(400),
                
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
                    .on_press_maybe(match validate_state(self).is_empty() {
                        true => Some(Message::FileDialog),
                        false => None
                    }),
            ]
            .align_x(Center)
            .padding(25)
            .spacing(10)
        )
    }
}