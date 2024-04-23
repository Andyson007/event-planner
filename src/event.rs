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
    pub host: serenity::User,
    pub title: String,
    pub description: String,
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
        host: serenity::User,
        location: String,
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
            title,
            description,
            start,
            end,
            host,
            location,
            members: HashSet::new(),
        })
    }

    pub fn addmember(&mut self, user: &User) {
        self.members.insert(user.clone());
    }

    pub fn removemember(&mut self, user: &User) {
        self.members.remove(user);
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "# {}\n## {}\n**Starts**: {:?}\n{}\n**Location**: {}\n**Members** ({}): {}",
            self.title,
            self.description,
            timeformat(self.start),
            match self.end {
                None => "".to_string(),
                Some(x) => format!("**Ends**: {}", timeformat(x)),
            },
            self.location,
            self.members.len(),
            self.members
                .iter()
                .map(|x| {
                    if *x == self.host {
                        format!("**{x}**")
                    } else {
                        format!("{x}")
                    }
                })
                .fold(String::new(), |sum, curr| format! {"{sum}\n- {curr}"})
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
