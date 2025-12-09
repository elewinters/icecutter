use iced::*;
use iced::widget::{*, column};

use crate::ui::Action;
use crate::ui::State;

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

    // if minutes and seconds exist and if they're both valid u32 integers that arent greater than 59, we return true
    match (minutes, seconds) {
        (Some(x), Some(y)) => {
            let Ok(minutes) = x.parse::<u32>() else {
                return false
            };

            let Ok(seconds) = y.parse::<u32>() else {
                return false
            };

            if minutes > 59 || seconds > 59 {
                return false
            }

            true
        }
        _ => false
    }
}

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

    // check if both timestamps are in the proper format
    // if one of these is empty it will also return true
    if !validate_timestamp(&state.from) {
        errors.push("'from' timestamp is not in a valid MM:SS format".to_owned());
    }

    if !validate_timestamp(&state.to) {
        errors.push("'to' timestamp is not in a valid MM:SS format".to_owned());
    }

    // check if "to" timestamp isn't 00:00
    if state.to == "00:00" || state.to == "0:00" || state.to == "00:0" || state.to == "0:0" {
        errors.push("'to' timestamp can't be 00:00".to_owned());
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