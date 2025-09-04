use iced::*;
use iced::widget::{*, column};

#[derive(Default)]
struct State {
    from: String,
    to: String,

    file: String,
    fps: String,
    convert_720p: bool
}

#[derive(Debug, Clone)]
enum Message {
    File(String),
    Fps(String),
    Convert(bool)
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
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
                    text_input("from", &self.from),
                    text("-")
                        .size(20),
                    text_input("to", &self.to)
                ]
                .spacing(10)
                .width(150),

                text_input("file name", &self.file)
                    .on_input(Message::File),
                
                text_input("fps", &self.fps)
                    .on_input(Message::Fps),

                checkbox("convert to 720p", self.convert_720p)
                    .on_toggle(Message::Convert)
            ]
            .align_x(Center)
            .padding(100)
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
        .run()
}