use ::serenity::all::{Mentionable, RoleId};
use chrono::DateTime;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    sync::{Arc, Mutex},
};

mod event;

use event::Event;

use crate::event::TIMEFORMAT;

struct Data {
    event: Arc<Mutex<Option<Event>>>,
    settings: Arc<Mutex<Settings>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command, owners_only)]
async fn set_notify_role(ctx: Context<'_>, role: Option<serenity::Role>) -> Result<(), Error> {
    if let Ok(mut x) = ctx.data().settings.lock() {
        x.notify_role = role.map(|x| x.id);
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("settings.json")
                .unwrap(),
        );
        serde_json::to_writer_pretty(&mut writer, &*x).unwrap();
        writer.flush().unwrap();
    }

    ctx.send(
        poise::CreateReply::default()
            .content("Success!")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, owners_only)]
async fn set_trusted_role(ctx: Context<'_>, role: Option<serenity::Role>) -> Result<(), Error> {
    if let Ok(mut x) = ctx.data().settings.lock() {
        x.trusted_role = role.map(|x| x.id);
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("settings.json")
                .unwrap(),
        );
        serde_json::to_writer_pretty(&mut writer, &*x).unwrap();
        writer.flush().unwrap();
    }

    ctx.send(
        poise::CreateReply::default()
            .content("Success!")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn cancel(ctx: Context<'_>) -> Result<(), Error> {
    let is_priviliged = {
        let future = if let Ok(x) = ctx.data().settings.lock() {
            x.trusted_role.map(|x| {
                ctx.author()
                    .has_role(ctx.http(), ctx.guild_id().unwrap(), x)
            })
        } else {
            None
        };
        match future {
            Some(x) => x.await.is_ok_and(|x| x),
            None => false,
        }
    };
    let is_creator = {
        if let Ok(x) = ctx.data().event.lock() {
            x.as_ref().is_some_and(|x| ctx.author() == &x.creator)
        } else {
            false
        }
    };
    if !is_priviliged && !is_creator {
        ctx.send(
            poise::CreateReply::default()
                .content("You aren't priviliged")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }
    let event = {
        let mut lock = match ctx.data().event.lock() {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };
        let event = lock.clone();
        *lock = None;
        event
    };
    if let Some(event) = event {
        ctx.say(format!(
            "{}\nThe event has been canceled",
            event.getmembers()
        ))
        .await?;
    } else {
        ctx.say("There is no event to cancel :(").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "A Title for the event"] title: String,
    #[description = "A description for the event"] description: String,
    #[description = "Format: %Y-%m-%d %H:%M"] start: String,
    #[description = "Format: %Y-%m-%d %H:%M"] end: Option<String>,
    #[description = "The location where everyone should meet at"] location: String,
    #[description = "Who hosts (used to inform everyone when that person changes plans)"]
    host: Option<serenity::User>,
) -> Result<(), Error> {
    let event = match Event::new(title, description, start, end, host, location, ctx.author()) {
        Ok(x) => x,
        Err(x) => {
            match x {
                event::Error::BadStart(x) => drop(ctx.reply(format!("Bad start: {x}")).await?),
                event::Error::BadEnd(x) => drop(ctx.reply(format!("Bad end: {x}")).await?),
                event::Error::Ambiguous => drop(ctx.reply("Time was ambiguous".to_string()).await?),
            };
            return Ok(());
        }
    };
    let mut response = format!("{}", event);
    if let Ok(x) = ctx.data().settings.lock() {
        if let Some(x) = x.notify_role {
            response = format!("{response}\nMention:\n{}", x.mention());
        }
    }
    let failed;
    if let Ok(mut lock) = ctx.data().event.lock() {
        *lock = Some(event);
        failed = false;
    } else {
        failed = true;
    }
    match failed {
        false => drop(ctx.reply(response).await?),
        true => drop(ctx.reply("An error occurred").await?),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, prefix_command)]
async fn update(
    ctx: Context<'_>,
    #[description = "A title for the event"] title: Option<String>,
    #[description = "A description for the event"] description: Option<String>,
    #[description = "Format: ISO 8601"] start: Option<String>,
    #[description = "Format: ISO 8601 (type None to remove)"] end: Option<String>,
    #[description = "The location where everyone should meet at"] location: Option<String>,
    #[description = "You can remove by setting removehost "] host: Option<serenity::User>,
    #[description = "This removes the host"] removehost: Option<bool>,
) -> Result<(), Error> {
    {
        let mut lock = match ctx.data().event.lock() {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };
        for i in lock.iter_mut() {
            if let Some(ref title) = title {
                i.title = title.clone();
            }
            if let Some(ref description) = description {
                i.description = description.clone();
            }
            if let Some(ref start) = start {
                i.start = DateTime::parse_from_str(start, TIMEFORMAT)?.into();
            }
            if let Some(ref end) = end {
                if end == "None" {
                    i.end = None
                } else {
                    i.end = Some(DateTime::parse_from_str(end, TIMEFORMAT)?.into());
                }
            }
            if let Some(ref location) = location {
                i.location = location.clone();
            }
            if removehost.is_some_and(|x| x) {
                i.host = None;
            } else if let Some(ref host) = host {
                i.host = Some(host.clone());
            }
        }
    }
    let lock = (match ctx.data().event.lock() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    })
    .clone();
    if let Some(x) = lock {
        let a = ctx.say("tmp").await?;
        a.edit(ctx, poise::CreateReply::default().content(format!("{}", x)))
            .await?;
    } else {
        ctx.reply("No event".to_string()).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let lock = (match ctx.data().event.lock() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    })
    .clone();
    if let Some(x) = lock {
        let a = ctx.say("tmp").await?;
        a.edit(ctx, poise::CreateReply::default().content(format!("{}", x)))
            .await?;
    } else {
        ctx.reply("No event".to_string()).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut lock = match ctx.data().event.lock() {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };
        for i in lock.iter_mut() {
            i.removemember(ctx.author());
        }
    }
    let lock = match ctx.data().event.lock() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    }
    .clone();
    if let Some(x) = lock {
        if Some(ctx.author()) == x.host.as_ref() {
            ctx.say(format!(
                "The host left!\n{}\n## Figure something out!",
                x.getmembers()
            ))
            .await?;
        } else {
            ctx.say("You left :(").await?;
        }
    } else {
        ctx.reply("No event".to_string()).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn remove(
    ctx: Context<'_>,
    #[description = "User to remove from event"] user: serenity::User,
) -> Result<(), Error> {
    let future = if let Ok(x) = ctx.data().settings.lock() {
        if let Some(x) = x.trusted_role {
            Some(
                ctx.author()
                    .has_role(ctx.http(), ctx.guild_id().unwrap(), x),
            )
        } else {
            return Ok(());
        }
    } else {
        None
    };
    let is_privileged = match future {
        Some(x) => x.await.is_ok_and(|x| x),
        None => false,
    };
    if !is_privileged {
        ctx.send(
            poise::CreateReply::default()
                .content("You aren't priviliged")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }
    {
        let mut lock = match ctx.data().event.lock() {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };
        for i in lock.iter_mut() {
            i.removemember(&user);
        }
    }
    let lock = match ctx.data().event.lock() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    }
    .clone();
    if let Some(x) = lock {
        if Some(user) == x.host {
            ctx.say(format!(
                "The host left!\n{}\n## Figure something out!",
                x.getmembers()
            ))
            .await?;
        } else {
            ctx.say("You left :(").await?;
        }
    } else {
        ctx.reply("No event".to_string()).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn add(
    ctx: Context<'_>,
    #[description = "Who are you adding?"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.unwrap_or(ctx.author().clone());
    {
        let mut lock = match ctx.data().event.lock() {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };
        for i in lock.iter_mut() {
            i.addmember(&user);
        }
    }
    ctx.reply(format!("Added {user}")).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().expect("Expected .env file");
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let settings: Settings = serde_json::from_reader(
        File::open("./settings.json").expect("Couldn't open file settings.json"),
    )
    .expect("Error reading json");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                create(),
                info(),
                add(),
                leave(),
                remove(),
                update(),
                cancel(),
                set_trusted_role(),
                set_notify_role(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    event: Arc::new(Mutex::new(None)),
                    settings: Arc::new(Mutex::new(settings)),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    client.start().await?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct Settings {
    trusted_role: Option<RoleId>,
    notify_role: Option<RoleId>,
}
