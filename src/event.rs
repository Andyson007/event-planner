use std::{collections::HashSet, fmt::Display};

use ::serenity::all::User;
use iso8601_timestamp::Timestamp;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone)]
pub struct Event {
    pub start: Timestamp,
    pub end: Option<Timestamp>,
    pub members: HashSet<serenity::User>,
    pub location: String,
    pub host: Option<serenity::User>,
    pub title: String,
    pub description: String,
    pub creator: serenity::User,
}

#[derive(Debug)]
pub enum Error {
    BadStart,
    BadEnd,
}

impl Event {
    pub fn new(
        title: String,
        description: String,
        start: String,
        end: Option<String>,
        host: Option<serenity::User>,
        location: String,
        creator: &serenity::User,
    ) -> Result<Self, Error> {
        let Some(start) = Timestamp::parse(&start) else {
            // ctx.reply("Bad format in start").await?;
            return Err(Error::BadStart);
        };
        let end = match end {
            Some(x) => match Timestamp::parse(&x) {
                None => {
                    return Err(Error::BadEnd);
                }
                x => x,
            },
            None => None,
        };
        Ok(Event {
            creator: creator.clone(),
            title,
            description,
            start,
            end,
            members: match host.clone() {
                Some(x) => HashSet::from([x]),
                None => HashSet::new(),
            },
            host,
            location,
        })
    }

    pub fn addmember(&mut self, user: &User) {
        self.members.insert(user.clone());
    }

    pub fn removemember(&mut self, user: &User) {
        self.members.remove(user);
    }

    pub fn getmembers(&self) -> String {
        self.members
            .iter()
            .map(|x| {
                if Some(x) == self.host.as_ref() {
                    format!("{x} (host)")
                } else {
                    format!("{x}")
                }
            })
            .fold(String::new(), |sum, curr| format! {"{sum}\n- {curr}"})
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "# {}\n## {}\n**Starts**: {:?}\n{}\n**Location**: {} {}\n**Members** ({}): {}",
            self.title,
            self.description,
            timeformat(self.start),
            match self.end {
                None => "".to_string(),
                Some(x) => format!("**Ends**: {}", timeformat(x)),
            },
            self.location,
            if let Some(x) = &self.host {
                format!("({x})")
            } else {
                "".to_string()
            },
            self.members.len(),
            self.getmembers()
        )
    }
}

fn timeformat(time: Timestamp) -> String {
    format!(
        "{} the {}th of {} ({}) at {}",
        time.weekday(),
        time.day(),
        time.month(),
        time.year(),
        time.time()
    )
}
