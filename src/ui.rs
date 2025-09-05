use iced::*;
use iced::widget::{*, column};

pub struct State {
    pub from: String,
    pub to: String,

    pub file: String,
    pub fps: String,
    pub convert_720p: bool
}

impl Default for State {
    fn default() -> Self {
        Self {
            from: String::default(),
            to: String::default(),

            file: String::default(),
            fps: String::from("60"),
            convert_720p: false
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFrom(String),
    ChangeTo(String),

    ChangeFile(String),
    ChangeFps(String),
    ChangeConvert720p(bool),

    ClickConvert,
}

fn validate_timestamp(timestamp: &str) -> bool {
    let split: Vec<&str> = timestamp.split(':').collect();

    let minutes = split.get(0);
    let seconds = split.get(1);

    match (minutes, seconds) {
        (Some(x), Some(y)) if x.parse::<u32>().is_ok() && y.parse::<u32>().is_ok() => true,
        _ => false
    }
}

fn validate_state(state: &State) -> bool {
    validate_timestamp(&state.from) &&
    validate_timestamp(&state.to) &&
    !state.file.is_empty() &&
    state.fps.parse::<u32>().is_ok()
}

impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ChangeFrom(from) => {
                self.from = from;
            }
            Message::ChangeTo(to) => {
                self.to = to;
            }
            Message::ChangeFile(file) => {
                self.file = file;
            }
            Message::ChangeFps(fps) => {
                self.fps = fps;
            }
            Message::ChangeConvert720p(convert) => {
                self.convert_720p = convert;
            }

            Message::ClickConvert => {
                super::convert(self);
            }
        }
    }

    pub fn view(&self) -> Container<'_, Message> {
        center(
            column![
                // text column
                column![
                    text("icecutter")
                        .size(25),
                    text("takes a video file, cuts it, and then compresses it with the specified configuration")
                ]
                .align_x(Center)
                .spacing(5)
                .padding(10),

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
                
                // file name
                container(
                    text_input("file name", &self.file)
                        .on_input(Message::ChangeFile),
                )
                .width(400),
                
                // fps, we set the width a bit lower
                container(
                    text_input("fps", &self.fps)
                        .on_input(Message::ChangeFps),
                )
                .width(75),
                
                // 720p checkbox
                checkbox("convert to 720p", self.convert_720p)
                    .on_toggle(Message::ChangeConvert720p),

                // convert button
                button("convert")
                    .on_press_maybe(match validate_state(self) {
                        true => Some(Message::ClickConvert),
                        false => None
                    })
            ]
            .align_x(Center)
            .padding(50)
            .spacing(10)
        )
    }
}