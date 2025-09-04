use iced::*;
use iced::widget::{*, column};

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Container<'_, Message> {
        container(
            column![
                button("Increment").on_press(Message::Increment),
                text(self.value).size(50),
                button("Decrement").on_press(Message::Decrement)
            ]
            .align_x(Center)
            .padding(10)
        )
        .width(Fill)
        .height(Fill)
        .align_x(Center)
        .align_y(Center)
    }
}

fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: (640.0, 480.0).into(),
        resizable: false,
        ..Default::default()
    };

    iced::application("counter", Counter::update, Counter::view)
        .window(window_settings)
        .run()
}