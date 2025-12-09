use std::error::Error;
use std::fmt;
use std::time::Duration;

// a time stamp with hours, minutes and seconds
// can be empty but not invalid
#[derive(Default, Clone, Debug)]
pub struct Timestamp {
    hours: Option<u8>,
    minutes: Option<u8>,
    seconds: Option<f32>
}

impl Timestamp {
    // make a new time stamp from a correctly formatted HH:MM:SS string
    // MM:SS or just SS is supported too
    pub fn new(timestamp: &str) -> Result<Timestamp, Box<dyn Error>> {
        // can't forget to trim!
        let timestamp = timestamp.trim();
        let split: Vec<&str> = timestamp.split(':').collect();

        // an empty timestamp is also valid
        if timestamp.is_empty() || split.is_empty() {
            return Ok(Timestamp {
                hours: None, 
                minutes: None, 
                seconds: None
            });
        }

        // if we have more than 3 colons
        if split.len() > 3 {
            return Err("a timestamp cannot have more than 3 values".into());
        }

        // get all the time values through a reversed loop
        let (hours, minutes, seconds) = {
            let mut hours = None;
            let mut minutes = None;
            let mut seconds = None;

            for (i, v) in split.iter().rev().enumerate() {
                match (i, v.is_empty()) {
                    (0, false) => seconds = Some(v.parse::<f32>()?.round()),
                    (1, false) => minutes = Some(v.parse::<u8>()?),
                    (2, false) => hours = Some(v.parse::<u8>()?),

                    (0, true) => seconds = Some(0.0),
                    (1, true) => minutes = Some(0),
                    (2, true) => hours = Some(0),

                    _ => unreachable!()
                }
            }

            (hours, minutes, seconds)
        };

        // verify validity
        if let Some(minutes) = minutes && minutes > 59 {
            return Err("a minute cannot be longer than 59 seconds".into());
        }

        if let Some(seconds) = seconds && seconds > 59.0 {
            return Err("a second cannot be longer than 59 seconds".into());
        }
        
        Ok(Timestamp { hours, minutes, seconds })
    }

    // gets the total amount of seconds that the timestamp represents
    pub fn total_secs(&self) -> f32 {
        (
            Duration::from_hours(self.hours.unwrap_or_default() as u64) + 
            Duration::from_mins(self.minutes.unwrap_or_default() as u64) + 
            Duration::from_secs(self.seconds.unwrap_or_default() as u64)
        ).as_secs_f32()
    }

    // check if the timestamp is empty
    pub fn is_empty(&self) -> bool {
        matches!((self.hours, self.minutes, self.seconds), (None, None, None))
    }
}

// this shouldn't be called on an empty timestamp
// check if it isn't empty by calling is_empty first
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.hours, self.minutes, self.seconds) {
            // dont show the hour if it's 0
            (Some(0), Some(min), Some(sec)) => write!(f, "{:02}:{:02}", min, sec),
            (Some(hour), Some(min), Some(sec)) => write!(f, "{hour}:{:02}:{:02}", min, sec),
            
            (None, Some(min), Some(sec)) => write!(f, "{:02}:{:02}", min, sec),
            (None, None, Some(sec)) => write!(f, "{:02}", sec),
            (None, None, None) => write!(f, ""),

            _ => unreachable!()
        }
    }
}