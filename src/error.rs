use std::fmt::Display;

// shows an error using an OS native dialog
pub fn show_error<T: Display>(error: T) {
    rfd::MessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(format!("{error}"))
        .set_level(rfd::MessageLevel::Error)
        .set_title("ERROR")
        .show();
}

// same as show_error, only with an AyncMessageDialog instead of a MessageDialog
pub async fn show_error_async<T: Display>(error: T) {
    rfd::AsyncMessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(format!("{error}"))
        .set_level(rfd::MessageLevel::Error)
        .set_title("ERROR")
        .show()
        .await;
}

// shows an error to the user synchronously
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        crate::error::show_error(format!($err))
    };
}

// returns a task that runs show_error_async
#[macro_export]
macro_rules! error_async {
    ($err:expr) => {
        iced::Task::perform(crate::error::show_error_async(format!($err)), |_| crate::ui::Message::None)
    };
}