use std::{collections::HashSet, fmt::Display};

use iso8601_timestamp::Timestamp;
use poise::serenity_prelude as serenity;
use ::serenity::all::User;

#[derive(Debug, Clone)]
pub struct Event {
    start: Timestamp,
    end: Option<Timestamp>,
    members: HashSet<serenity::User>,
    location: String,
    host: serenity::User,
    title: String,
    description: String,
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
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "# {}\n## {}\n## Starts: {:?}\n{}\n**Location**: {}\n**Members**: {}",
            self.title,
            self.description,
            self.start.to_iso_week_date(),
            match self.end {
                None => "".to_string(),
                Some(x) => format!("## Ends: {x}"),
            },
            self.location,
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
