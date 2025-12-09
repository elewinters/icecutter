use iced::{Element, Color};
use iced::widget::{*, column};

use crate::ui::Action;
use crate::ui::State;

// checks if all of the input values are valid, and returns a vector of string errors
// if the state is valid, then an empty vector is returned
pub fn validate_state(state: &State) -> Vec<String> {
    let mut errors = Vec::new();

    // check if only one of the timestamps is filled and throw an error
    if state.to.is_empty() && !state.from.is_empty() {
        errors.push("'to' timestamp is empty, while the 'from' timestamp is not".to_owned());
    }

    if state.from.is_empty() && !state.to.is_empty() {
        errors.push("'from' timestamp is empty, while the 'to' timestamp is not".to_owned());
    }

    // check if "to" timestamp isn't 00:00
    if !state.to.is_empty() && state.to.total_secs() == 0.0 {
        errors.push("'to' timestamp can't be 0".to_owned());
    }

    // check if "from" is bigger than "to"
    if state.from.total_secs() > state.to.total_secs() {
        errors.push("'from' timestamp is longer than the 'to' timestamp".to_owned());
    }

    // check if the timestamps are the same
    if state.from.total_secs() == state.to.total_secs() {
        errors.push("the 'from' and 'to' timestamps cannot be the same".to_owned());
    }

    // check if input file exists
    if !state.file.exists() {
        errors.push("input file does not exist".to_owned());
    }

    // check if fps field is a valid number
    if state.fps.parse::<u32>().is_err() {
        errors.push("'fps' field is not a valid unsigned integer".to_owned())
    }

    errors
}

// view responsible for showing error messages above the convert button
pub fn error_view(state: &State) -> Element<'_, Action> {
    let errors = validate_state(state);
    
    if errors.is_empty() {
        return space().into();
    }

    column(
        errors.into_iter()
            .map(|error| {
                text(error)
                    .color(Color::from_rgb(1.0, 0.0, 0.0))
                    .size(12)
                    .into()
            })
            .collect::<Vec<Element<Action>>>()
    )
    .spacing(5)
    .into()
}