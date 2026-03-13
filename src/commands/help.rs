use crate::{Context, Error};

#[poise::command(slash_command, user_cooldown = 10)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to get help for"] command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            show_subcommands: true,
            ephemeral: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
