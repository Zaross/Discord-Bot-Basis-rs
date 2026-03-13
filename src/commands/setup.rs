use crate::{ctx_ext::CtxTranslate, Context, Error};

#[poise::command(slash_command, description_localized("de", "Richte den Bot für diesen Server ein"))]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(ctx.t("setup.starting")).await?;
    Ok(())
}