use crate::{ctx_ext::CtxTranslate, permissions, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    user_cooldown = 10,
    guild_only,
    check = "permissions::guild_only"
)]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild()
        .ok_or("Not in a guild")?
        .clone();

    let owner = guild.owner_id.to_user(ctx).await?;

    let created_at = guild.id.created_at().format("%Y-%m-%d %H:%M UTC").to_string();

    let member_count = guild.member_count;

    let boost_level = match guild.premium_tier {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };
    let boost_count = guild.premium_subscription_count.unwrap_or(0);

    let embed = serenity::CreateEmbed::default()
        .title(ctx.tv("serverinfo.title", &[("name", &guild.name)]))
        .thumbnail(
            guild
                .icon_url()
                .unwrap_or_else(|| String::new()),
        )
        .field(ctx.t("serverinfo.id"), guild.id.to_string(), true)
        .field(ctx.t("serverinfo.owner"), format!("<@{}>", owner.id), true)
        .field(
            ctx.t("serverinfo.members"),
            member_count.to_string(),
            true,
        )
        .field(
            ctx.t("serverinfo.created"),
            created_at,
            false,
        )
        .field(
            ctx.t("serverinfo.boost_level"),
            format!("Level {} ({} boosts)", boost_level, boost_count),
            true,
        )
        .color(serenity::Colour::DARK_GREEN);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
