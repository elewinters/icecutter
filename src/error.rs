use std::fmt::Display;

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