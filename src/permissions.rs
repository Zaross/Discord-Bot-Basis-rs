use crate::{Context, Error};

pub fn is_owner(ctx: Context<'_>) -> bool {
    ctx.author().id == ctx.data().owner_id
}

/// Usage:
/// #[poise::command(slash_command, check = "permissions::owner_only")]
/// pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> { … }
pub async fn owner_only(ctx: Context<'_>) -> Result<bool, Error> {
    if is_owner(ctx) {
        return Ok(true);
    }
    ctx.say(ctx.data().translator.get(
        ctx.locale().unwrap_or("en"),
        "errors.owner_only",
    ))
    .await?;
    Ok(false)
}

pub async fn guild_only(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.guild_id().is_some() {
        return Ok(true);
    }
    ctx.say(ctx.data().translator.get(
        ctx.locale().unwrap_or("en"),
        "errors.not_in_guild",
    ))
    .await?;
    Ok(false)
}
