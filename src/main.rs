mod cache;
mod commands;
mod ctx_ext;
mod db;
mod health;
pub mod i18n;
mod logging;
mod metrics;
mod permissions;

use anyhow::Context as _;
use cache::{GuildCache, GuildSettings};
use db::Database;
use i18n::Translator;
use metrics::Metrics;
use poise::serenity_prelude as serenity;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tracing::{error, info, warn};

pub mod tasks;

pub struct Data {
    pub owner_id: serenity::UserId,
    pub support_server_id: Option<serenity::GuildId>,
    pub start_time: std::time::Instant,
    pub translator: Arc<Translator>,
    pub db: Database,
    pub cache: GuildCache,
    pub metrics: Arc<Metrics>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let bot_name = std::env::var("BOT_NAME").unwrap_or_else(|_| "Bot".to_string());
    let bot_folder = PathBuf::from(&bot_name);
    let logs_folder = bot_folder.join("logs");

    std::fs::create_dir_all(&logs_folder)
        .with_context(|| format!("Failed to create '{}'", logs_folder.display()))?;

    logging::setup(&logs_folder);

    info!("Starting {} …", bot_name);
    info!("Logs → {}/logs/", bot_name);

    let token = std::env::var("TOKEN").context("Missing TOKEN")?;

    let owner_id: u64 = std::env::var("OWNER_ID")
        .context("Missing OWNER_ID")?
        .parse()
        .context("OWNER_ID must be a u64")?;

    let support_server_id: Option<u64> = std::env::var("SUPPORT_SERVER")
        .ok()
        .and_then(|s| s.parse().ok());

    let shard_count: ShardCount = std::env::var("SHARD_COUNT")
        .unwrap_or_default()
        .parse()
        .unwrap_or(ShardCount::Auto);

    let metrics = Metrics::new().context("Failed to create metrics registry")?;

    let db_url_raw = std::env::var("DATABASE_URL").ok();
    let db_url = db::resolve_database_url(db_url_raw.as_deref(), &bot_folder)
        .context("Invalid DATABASE_URL")?;

    info!("Database → {}", db_url);

    let database = Database::connect(&db_url).await?;
    let db_pool_for_cron = database.pool.clone();

    let translator = Arc::new(
        Translator::load("locales").context("Failed to load locales/")?
    );

    let data = Data {
        owner_id: serenity::UserId::new(owner_id),
        support_server_id: support_server_id.map(serenity::GuildId::new),
        start_time: std::time::Instant::now(),
        translator: translator.clone(),
        db: database.clone(),
        cache: GuildCache::new(),
        metrics: metrics.clone(),
    };

    tokio::spawn(health::start_health_server(metrics.clone()));

    {
        let m = metrics.clone();
        let start = data.start_time;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                m.uptime_seconds.set(start.elapsed().as_secs_f64());
            }
        });
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            on_error: |err| Box::pin(on_error(err)),
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        cmd = ctx.command().name,
                        user = %ctx.author().tag(),
                        guild = ?ctx.guild_id().map(|g| g.to_string()),
                        "command invoked"
                    );

                    ctx.data()
                        .metrics
                        .commands_total
                        .with_label_values(&[ctx.command().name.as_str()])
                        .inc();

                    if let Some(guild_id) = ctx.guild_id() {
                        let data = ctx.data();
                        if data.cache.get(guild_id).is_none() {
                            load_guild_settings(ctx, guild_id).await;
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let data = data;
            let db_pool_for_cron = db_pool_for_cron.clone();

            Box::pin(async move {
                info!("Logged in as {}", ready.user.tag());

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Slash commands registered globally.");

                let http = ctx.http.clone();
                let translator_for_cron = data.translator.clone();

                tokio::spawn(tasks::cronjobs::start_all(
                    http,
                    db_pool_for_cron,
                    translator_for_cron,
                ));

                info!("Cronjob handler started.");
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

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        warn!("Shutdown signal received — closing shards…");
        shard_manager.shutdown_all().await;
        info!("All shards closed.");
    });

    info!("Connecting to Discord… (shards: {})", shard_count);

    match shard_count {
        ShardCount::Auto => client.start_autosharded().await,
        ShardCount::Fixed(n) => client.start_shards(n).await,
    }
    .context("Client error")?;

    database.pool.close().await;
    info!("Database pool closed. Goodbye!");

    Ok(())
}

async fn load_guild_settings(ctx: Context<'_>, guild_id: serenity::GuildId) {
    let data = ctx.data();
    let id_str = guild_id.to_string();

    let result = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT language, log_channel_id FROM guild_settings WHERE guild_id = ?",
    )
    .bind(&id_str)
    .fetch_optional(&data.db.pool)
    .await;

    match result {
        Ok(Some((language, log_channel_id_str))) => {
            data.cache.set(
                guild_id,
                GuildSettings {
                    language,
                    log_channel_id: log_channel_id_str.and_then(|s| s.parse::<u64>().ok()),
                },
            );
        }
        Ok(None) => {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO guild_settings (guild_id) VALUES (?)",
            )
            .bind(&id_str)
            .execute(&data.db.pool)
            .await;

            data.cache.set(guild_id, GuildSettings::default());
        }
        Err(e) => error!("Failed to load guild settings for {}: {}", guild_id, e),
    }
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!("Failed to start bot: {:?}", error);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!(
                cmd = ctx.command().name,
                user = %ctx.author().tag(),
                "command error: {:?}",
                error
            );

            ctx.data()
                .metrics
                .command_errors_total
                .with_label_values(&[ctx.command().name.as_str()])
                .inc();

            let msg = ctx
                .data()
                .translator
                .get(ctx.locale().unwrap_or("en"), "errors.generic");

            let _ = ctx.say(msg).await;
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            let seconds = remaining_cooldown.as_secs().max(1).to_string();

            let msg = ctx.data().translator.get_with(
                ctx.locale().unwrap_or("en"),
                "cooldown.hit",
                &[("seconds", &seconds)],
            );

            let _ = ctx
                .send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Unhandled framework error: {}", e);
            }
        }
    }
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => info!("Received SIGTERM"),
            _ = sigint.recv() => info!("Received SIGINT"),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.expect("Ctrl+C handler");
        info!("Received Ctrl+C");
    }
}

#[derive(Debug, Clone, Copy)]
enum ShardCount {
    Auto,
    Fixed(u32),
}

impl std::fmt::Display for ShardCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardCount::Auto => write!(f, "auto"),
            ShardCount::Fixed(n) => write!(f, "{}", n),
        }
    }
}

impl std::str::FromStr for ShardCount {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "" | "auto" => Ok(ShardCount::Auto),
            n => n.parse::<u32>().map(ShardCount::Fixed).map_err(|_| ()),
        }
    }
}