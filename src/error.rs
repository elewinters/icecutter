use std::fmt::Display;
use rfd::*;

// shows an error using an OS native dialog
pub fn show_error(error: impl Display) {
    MessageDialog::new()
        .set_buttons(MessageButtons::Ok)
        .set_description(error.to_string())
        .set_level(MessageLevel::Error)
        .set_title("ERROR")
        .show();
}

// same as show_error, only with an AyncMessageDialog instead of a MessageDialog
pub async fn show_error_async(error: impl Display) {
    AsyncMessageDialog::new()
        .set_buttons(MessageButtons::Ok)
        .set_description(error.to_string())
        .set_level(MessageLevel::Error)
        .set_title("ERROR")
        .show()
        .await;
}

// shows an error to the user synchronously
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        $crate::error::show_error(format!($err))
    };
}

// returns a task that runs show_error_async
#[macro_export]
macro_rules! error_async {
    ($err:expr) => {
        iced::Task::perform($crate::error::show_error_async(format!($err)), |_| $crate::ui::Message::None)
    };
}