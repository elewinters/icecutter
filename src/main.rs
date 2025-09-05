use iced::*;
use iced::widget::{*, column};

struct State {
    from: String,
    to: String,

    file: String,
    fps: String,
    convert_720p: bool
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
enum Message {
    From(String),
    To(String),

    File(String),
    Fps(String),
    Convert(bool),
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::From(from) => {
                self.from = from;
            }
            Message::To(to) => {
                self.to = to;
            }

            Message::File(file) => {
                self.file = file;
            }
            Message::Fps(fps) => {
                self.fps = fps;
            }
            Message::Convert(convert) => {
                self.convert_720p = convert;
            }
        }
    }

    fn view(&self) -> Container<'_, Message> {
        center(
            column![
                column![
                    text("icecutter")
                        .size(25),
                    text("takes a video file, cuts it, and then compresses it with the specified configuration")
                ]
                .align_x(Center)
                .spacing(5)
                .padding(10),

                row![
                    text_input("from", &self.from)
                        .on_input(Message::From),
                    text("-")
                        .size(20),
                    text_input("to", &self.to)
                        .on_input(Message::To),
                ]
                .spacing(10)
                .width(150),

                text_input("file name", &self.file)
                    .on_input(Message::File),
                
                text_input("fps", &self.fps)
                    .on_input(Message::Fps),

                checkbox("convert to 720p", self.convert_720p)
                    .on_toggle(Message::Convert),

                button("convert")
            ]
            .align_x(Center)
            .padding(50)
            .spacing(10)
        )
    }
}

fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: (640.0, 480.0).into(),
        resizable: false,
        ..Default::default()
    };

    iced::application("icecutter", State::update, State::view)
        .window(window_settings)
        .run_with(|| {
            (State::default(), Task::none())
        })
}