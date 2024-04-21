use poise::serenity_prelude as serenity;
use std::{
    env,
    sync::{Arc, Mutex},
};

struct Data {
    event: Arc<Mutex<Event>>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Event {
    val: i32,
    // start: u64,
    // end: Option<u64>,
    // members: Vec<serenity::User>,
}

#[poise::command(slash_command, prefix_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "A title for the event"] title: String,
    #[description = "A description for the event"] description: String,
    #[description = "When does the event start?"] start: String,
    #[description = "When does the event end? (Optional)"] end: String,
) -> Result<(), Error> {
    let u = ctx.author();
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Expected .env file");
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    event: Arc::new(Mutex::new(Event { val: 5 })),
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
