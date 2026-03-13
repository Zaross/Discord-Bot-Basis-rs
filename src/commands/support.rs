use crate::{ctx_ext::CtxTranslate, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, description_localized("de", "Link zum Support-Server erhalten"))]
pub async fn support(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.data().support_server_id else {
        ctx.say(ctx.t("support.no_server")).await?;
        return Ok(());
    };

    let channels = guild_id.channels(ctx).await?;
    let invite_channel = channels
        .values()
        .find(|c| matches!(c.kind, serenity::ChannelType::Text));

    let Some(channel) = invite_channel else {
        ctx.say(ctx.t("support.no_channel")).await?;
        return Ok(());
    };

    let invite = channel
        .create_invite(
            ctx,
            serenity::CreateInvite::default()
                .max_age(86400)
                .max_uses(1)
                .unique(true),
        )
        .await?;

    let embed = serenity::CreateEmbed::default()
        .title(ctx.t("support.title"))
        .description(ctx.tv("support.description", &[("code", &invite.code)]))
        .color(serenity::Colour::DARK_GREEN);

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
