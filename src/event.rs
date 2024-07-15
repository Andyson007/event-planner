use std::{collections::HashSet, fmt::Display};

use ::serenity::all::User;
use chrono::{offset::LocalResult, DateTime, Local, NaiveDateTime, ParseError, TimeZone};
use poise::serenity_prelude as serenity;

pub const TIMEFORMAT: &str = "%Y-%m-%d %H:%M";

#[derive(Debug, Clone)]
pub struct Event {
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
    pub members: HashSet<serenity::User>,
    pub location: String,
    pub host: Option<serenity::User>,
    pub title: String,
    pub description: String,
    pub creator: serenity::User,
}

// This hould really have start, end
// with each of them having fields from a
// different enum
#[derive(Debug)]
pub enum Error {
    BadStart(ParseError),
    BadEnd(ParseError),
    Ambiguous,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "err")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    // fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
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
        let naivestart =
            NaiveDateTime::parse_from_str(&start, TIMEFORMAT).map_err(Error::BadStart)?;
        let LocalResult::Single(start) = Local.from_local_datetime(&naivestart) else {
            return Err(Error::Ambiguous);
        };
        // DateTime::<Local>::from_naive_utc_and_offset(naivestart, *Local::now().offset());
        let end = match end {
            Some(x) => {
                let naiveend =
                    NaiveDateTime::parse_from_str(&x, TIMEFORMAT).map_err(Error::BadEnd)?;
                let LocalResult::Single(end) = Local.from_local_datetime(&naiveend) else {
                    return Err(Error::Ambiguous);
                };
                Some(end)
            }

            None => None,
        };
        println!("{start}");
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
            "# {}\n## {}\n**Starts**: <t:{}:R> (<t:{}:F>)\n{}\n**Location**: {} {}\n**Members** ({}): {}",
            self.title,
            self.description,
            self.start.to_utc().timestamp(),
            match self.end {
                None => "".to_string(),
                Some(x) => format!(
                    "**Ends**: <t:{}:R> (<t:{}:F>)",
                    x.to_utc().timestamp(),
                    x.to_utc().timestamp()
                ),
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
