use crate::{db::Backend, permissions, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, check = "permissions::owner_only", ephemeral)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let uptime = data.start_time.elapsed();

    let db_backend = match data.db.backend {
        Backend::Sqlite => "SQLite",
        Backend::Postgres => "Postgres",
    };

    let guild_count = ctx.cache().guild_count();
    let cache_size = data.cache.len();

    let embed = serenity::CreateEmbed::default()
        .title("⚙️ Bot Status")
        .field("Uptime", format_duration(uptime), true)
        .field("Database", db_backend, true)
        .field("Guilds", guild_count.to_string(), true)
        .field("Cache entries", cache_size.to_string(), true)
        .field("Metrics", "http://localhost:5000/metrics", false)
        .color(serenity::Colour::GOLD);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, check = "permissions::owner_only", ephemeral)]
pub async fn sql(
    ctx: Context<'_>,
    #[description = "SQL SELECT query to execute"] query: String,
) -> Result<(), Error> {
    let trimmed = query.trim().to_uppercase();
    if !trimmed.starts_with("SELECT") {
        ctx.say("❌ Only SELECT queries are allowed.").await?;
        return Ok(());
    }

    let start = std::time::Instant::now();
    let rows = sqlx::query(&query)
        .fetch_all(&ctx.data().db.pool)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let elapsed = start.elapsed().as_millis();

    if rows.is_empty() {
        ctx.say(format!("✅ No rows returned. ({} ms)", elapsed))
            .await?;
        return Ok(());
    }

    let result = format!("{} row(s) in {} ms", rows.len(), elapsed);
    ctx.say(format!("```\n{}\n```", result)).await?;
    Ok(())
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else {
        format!("{}m {}s", minutes, seconds)
    }
}
