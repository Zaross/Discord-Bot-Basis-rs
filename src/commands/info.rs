use crate::{ctx_ext::CtxTranslate, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, description_localized("de", "Zeigt Informationen über den Bot"))]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = ctx.data().start_time.elapsed();
    let uptime_str = format_duration(uptime);

    let bot_user = ctx.cache().current_user().clone();
    let guild_count = ctx.cache().guild_count();

    let embed = serenity::CreateEmbed::default()
        .title(ctx.t("info.title"))
        .thumbnail(
            bot_user
                .avatar_url()
                .unwrap_or_else(|| bot_user.default_avatar_url()),
        )
        .field(ctx.t("info.uptime"), &uptime_str, true)
        .field(ctx.t("info.servers"), guild_count.to_string(), true)
        .field(ctx.t("info.version"), env!("CARGO_PKG_VERSION"), true)
        .field(
            ctx.t("info.owner"),
            format!("<@{}>", ctx.data().owner_id),
            true,
        )
        .color(serenity::Colour::BLITZ_BLUE);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
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
