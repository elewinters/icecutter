use std::error::Error;
use std::fmt;
use std::time::Duration;

// a time stamp with hours, minutes and seconds
// can be empty but not invalid
pub struct Timestamp {
    hours: Option<u64>,
    minutes: Option<u8>,
    seconds: Option<u8>
}

impl Timestamp {
    // make a new time stamp from a correctly formatted HH:MM:SS string
    // MM:SS or just SS is supported too
    pub fn new(timestamp: &str) -> Result<Timestamp, Box<dyn Error>> {
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
                match i {
                    0 => seconds = Some(v.parse::<u8>()?),
                    1 => minutes = Some(v.parse::<u8>()?),
                    2 => hours = Some(v.parse::<u64>()?),
                    _ => unreachable!()
                }
            }

            (hours, minutes, seconds)
        };

        // verify validity
        if let Some(minutes) = minutes && minutes > 59 {
            return Err("a minute cannot be longer than 59 seconds".into());
        }

        if let Some(seconds) = seconds && seconds > 59 {
            return Err("a second cannot be longer than 59 seconds".into());
        }
        
        println!("hours: {:?}, minutes: {:?}, seconds: {:?}", hours, minutes, seconds);

        Ok(Timestamp { hours, minutes, seconds })
    }

    // gets the total amount of seconds that the timestamp represents
    pub fn total_secs(&self) -> f32 {
        (
            Duration::from_hours(self.hours.unwrap_or_default()) + 
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
            (Some(hour), Some(min), Some(sec)) => write!(f, "{hour}:{:02}:{:02}", min, sec),
            (None, Some(min), Some(sec)) => write!(f, "{:02}:{:02}", min, sec),
            (None, None, Some(sec)) => write!(f, "{:02}", sec),
            (None, None, None) => write!(f, ""),

            _ => unreachable!()
        }
    }
}