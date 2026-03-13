mod commands;
mod ctx_ext;
mod health;
pub mod i18n;

use anyhow::Context as _;
use i18n::Translator;
use poise::serenity_prelude as serenity;
use tracing::{error, info};

pub struct Data {
    pub owner_id: serenity::UserId,
    pub support_server_id: Option<serenity::GuildId>,
    pub start_time: std::time::Instant,
    pub translator: Translator,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    setup_logging();

    info!("Starting Discord bot...");

    let token = std::env::var("TOKEN").context("Missing TOKEN environment variable")?;
    let owner_id: u64 = std::env::var("OWNER_ID")
        .context("Missing OWNER_ID environment variable")?
        .parse()
        .context("OWNER_ID must be a valid u64")?;
    let support_server_id: Option<u64> = std::env::var("SUPPORT_SERVER")
        .ok()
        .and_then(|s| s.parse().ok());

    let translator = Translator::load("locales")
        .context("Failed to load locale files — make sure the 'locales/' directory exists")?;

    let data = Data {
        owner_id: serenity::UserId::new(owner_id),
        support_server_id: support_server_id.map(serenity::GuildId::new),
        start_time: std::time::Instant::now(),
        translator,
    };

    tokio::spawn(health::start_health_server());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            on_error: |err| Box::pin(on_error(err)),
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Command '{}' invoked by {}",
                        ctx.command().name,
                        ctx.author().tag()
                    );
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", ready.user.tag());
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Slash commands registered globally.");
                Ok(data)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to build serenity client")?;

    info!("Connecting to Discord...");
    client.start().await.context("Client error")?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!("Failed to start bot: {:?}", error);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command '{}': {:?}", ctx.command().name, error);
            let _ = ctx
                .say("An error occurred while executing this command.")
                .await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

fn setup_logging() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    std::fs::create_dir_all("Logs").ok();

    let file_appender = tracing_appender::rolling::daily("Logs", "bot.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    Box::leak(Box::new(_guard));

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,serenity=warn,poise=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).pretty())
        .with(fmt::layer().with_target(false).with_writer(non_blocking))
        .init();
}
