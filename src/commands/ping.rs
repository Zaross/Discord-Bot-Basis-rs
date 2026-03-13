use crate::{ctx_ext::CtxTranslate, Context, Error};

#[poise::command(
    slash_command,
    user_cooldown = 5,
    description_localized("de", "Überprüfe die Antwortzeit des Bots")
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say(ctx.t("ping.pinging")).await?;
    let elapsed = start.elapsed().as_millis();

    msg.edit(
        ctx,
        poise::CreateReply::default()
            .content(ctx.tv("ping.pong", &[("ms", &elapsed.to_string())])),
    )
    .await?;
    Ok(())
}
