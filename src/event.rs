use std::{collections::HashSet, fmt::Display};

use ::serenity::all::User;
use chrono::{NaiveDateTime, ParseError};
use poise::serenity_prelude as serenity;

pub const TIMEFORMAT: &str = "%Y-%m-%d %H:%M";

#[derive(Debug, Clone)]
pub struct Event {
    pub start: NaiveDateTime,
    pub end: Option<NaiveDateTime>,
    pub members: HashSet<serenity::User>,
    pub location: String,
    pub host: Option<serenity::User>,
    pub title: String,
    pub description: String,
    pub creator: serenity::User,
}

#[derive(Debug)]
pub enum Error {
    BadStart(ParseError),
    BadEnd(ParseError),
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
        let start = NaiveDateTime::parse_from_str(&start, TIMEFORMAT).map_err(Error::BadStart)?;
        let end = match end {
            Some(x) => {
                Some(NaiveDateTime::parse_from_str(&x, TIMEFORMAT).map_err(Error::BadEnd)?)
            }

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
            self.start,
            match self.end {
                None => "".to_string(),
                Some(x) => format!("**Ends**: {}", x),
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
