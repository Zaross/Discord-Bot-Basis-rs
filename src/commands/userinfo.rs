use crate::{ctx_ext::CtxTranslate, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, user_cooldown = 5, guild_only)]
pub async fn userinfo(
    ctx: Context<'_>,
    #[description = "The user to look up (defaults to yourself)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or_else(|| ctx.author());

    let member = if let Some(guild_id) = ctx.guild_id() {
        guild_id.member(ctx, user.id).await.ok()
    } else {
        None
    };

    let created_at = user.created_at().format("%Y-%m-%d %H:%M UTC").to_string();

    let joined_at = member
        .as_ref()
        .and_then(|m| m.joined_at)
        .map(|ts| ts.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let nick = member
        .as_ref()
        .and_then(|m| m.nick.clone())
        .unwrap_or_else(|| "-".to_string());

    let roles: String = member
        .as_ref()
        .map(|m| {
            let role_mentions: Vec<String> = m.roles.iter().map(|r| format!("<@&{}>", r)).collect();
            if role_mentions.is_empty() {
                "-".to_string()
            } else {
                role_mentions.join(", ")
            }
        })
        .unwrap_or_else(|| "-".to_string());

    let embed = serenity::CreateEmbed::default()
        .title(ctx.tv("userinfo.title", &[("user", &user.tag())]))
        .thumbnail(user.face())
        .field(ctx.t("userinfo.id"), user.id.to_string(), true)
        .field(ctx.t("userinfo.nickname"), nick, true)
        .field(ctx.t("userinfo.account_created"), created_at, false)
        .field(ctx.t("userinfo.joined_server"), joined_at, false)
        .field(ctx.t("userinfo.roles"), roles, false)
        .color(serenity::Colour::BLITZ_BLUE)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "ID: {}",
            user.id
        )));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}
