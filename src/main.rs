use poise::serenity_prelude as serenity;
use std::{
    env,
    sync::{Arc, Mutex},
};

mod event;

use event::Event;

struct Data {
    event: Arc<Mutex<Option<Event>>>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "A title for the event"] title: String,
    #[description = "A description for the event"] description: String,
    #[description = "Format: ISO 8601"] start: String,
    #[description = "Format: ISO 8601"] end: Option<String>,
    #[description = "The location where everyone should meet at"] location: String,
    #[description = "Who hosts (used to inform everyone when that person changes plans)"]
    host: Option<serenity::User>,
) -> Result<(), Error> {
    let u = ctx.author();
    let event = match Event::new(
        title,
        description,
        start,
        end,
        host.unwrap_or(u.clone()),
        location,
    ) {
        Ok(x) => x,
        Err(x) => {
            match x {
                event::Error::BadStart => drop(ctx.reply("Bad start").await?),
                event::Error::BadEnd => drop(ctx.reply("Bad end").await?),
            };
            return Ok(());
        }
    };
    let response = format!("{:#?}", event);
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

#[poise::command(slash_command, prefix_command)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let lock = (match ctx.data().event.lock() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    })
    .clone();
    if let Some(x) = lock {
        ctx.reply(format!("{}", x)).await?;
    } else {
        ctx.reply(format!("No event")).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Expected .env file");
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create(), info()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    event: Arc::new(Mutex::new(None)),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();
    client.start().await.unwrap();
}
