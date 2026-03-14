use poise::serenity_prelude as serenity;
use crate::cache::GuildSettings;
use crate::{Context, Error};

const SETUP_TIMEOUT_SECS: u64 = 120;

fn locale_display(code: &str) -> String {
    match code {
        "de" => "🇩🇪  Deutsch".to_string(),
        "en" => "🇬🇧  English".to_string(),
        "fr" => "🇫🇷  Français".to_string(),
        "es" => "🇪🇸  Español".to_string(),
        "it" => "🇮🇹  Italiano".to_string(),
        "nl" => "🇳🇱  Nederlands".to_string(),
        "pl" => "🇵🇱  Polski".to_string(),
        "pt" => "🇵🇹  Português".to_string(),
        "ru" => "🇷🇺  Русский".to_string(),
        "tr" => "🇹🇷  Türkçe".to_string(),
        "ja" => "🇯🇵  日本語".to_string(),
        "zh" => "🇨🇳  中文".to_string(),
        "ko" => "🇰🇷  한국어".to_string(),
        other => format!("🌐  {}", other.to_uppercase()),
    }
}

#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    description_localized("de", "Richtet den Bot für diesen Server ein")
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let initial_locale = ctx.locale().unwrap_or("en");
    let t = &ctx.data().translator;

    let lang_options: Vec<serenity::CreateSelectMenuOption> = {
        let mut locales = t.available_locales();
        locales.sort();
        locales
            .into_iter()
            .map(|code| serenity::CreateSelectMenuOption::new(locale_display(code), code))
            .collect()
    };

    let lang_select = serenity::CreateSelectMenu::new(
        "setup_language",
        serenity::CreateSelectMenuKind::String { options: lang_options },
    )
    .placeholder(t.get(initial_locale, "setup.placeholders.language"))
    .min_values(1)
    .max_values(1);

    let log_select = serenity::CreateSelectMenu::new(
        "setup_log_channel",
        serenity::CreateSelectMenuKind::Channel {
            channel_types: Some(vec![serenity::ChannelType::Text]),
            default_channels: None,
        },
    )
    .placeholder(t.get(initial_locale, "setup.placeholders.log_channel"))
    .min_values(0)
    .max_values(1);

    let confirm_btn = serenity::CreateButton::new("setup_confirm")
        .label(t.get(initial_locale, "setup.buttons.save"))
        .style(serenity::ButtonStyle::Success)
        .emoji('✅');

    let cancel_btn = serenity::CreateButton::new("setup_cancel")
        .label(t.get(initial_locale, "setup.buttons.cancel"))
        .style(serenity::ButtonStyle::Danger)
        .emoji('❌');

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(t.get(initial_locale, "setup.start"))
                .components(vec![
                    serenity::CreateActionRow::SelectMenu(lang_select),
                    serenity::CreateActionRow::SelectMenu(log_select),
                    serenity::CreateActionRow::Buttons(vec![confirm_btn, cancel_btn]),
                ])
                .ephemeral(true),
        )
        .await?;

    let mut lang = if t.has_locale(initial_locale) {
        initial_locale.split('-').next().unwrap_or("en").to_string()
    } else {
        "en".to_string()
    };

    let mut log_channel_id: Option<String> = None;

    let msg = reply.message().await?;

    loop {
        let interaction = msg
            .await_component_interaction(ctx.serenity_context())
            .author_id(ctx.author().id)
            .timeout(std::time::Duration::from_secs(SETUP_TIMEOUT_SECS))
            .await;

        match interaction {
            None => {
                reply
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .content(t.get(&lang, "setup.timeout"))
                            .components(vec![]),
                    )
                    .await?;
                return Ok(());
            }
            Some(i) => match i.data.custom_id.as_str() {
                "setup_cancel" => {
                    i.defer_ephemeral(ctx.http()).await?;
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content(t.get(&lang, "setup.cancelled"))
                                .components(vec![]),
                        )
                        .await?;
                    return Ok(());
                }
                "setup_confirm" => {
                    i.defer_ephemeral(ctx.http()).await?;
                    break;
                }
                "setup_language" => {
                    if let serenity::ComponentInteractionDataKind::StringSelect { values } = &i.data.kind {
                        if let Some(v) = values.first() {
                            lang = v.clone();
                        }
                    }
                    i.defer_ephemeral(ctx.http()).await?;
                }
                "setup_log_channel" => {
                    if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &i.data.kind {
                        log_channel_id = values.first().map(|id| id.to_string());
                    }
                    i.defer_ephemeral(ctx.http()).await?;
                }
                _ => {
                    i.defer_ephemeral(ctx.http()).await?;
                }
            },
        }
    }

    let db = &ctx.data().db.pool;

    sqlx::query("INSERT OR IGNORE INTO guild_settings (guild_id) VALUES (?)")
        .bind(&guild_id)
        .execute(db)
        .await?;

    sqlx::query("UPDATE guild_settings SET language = ? WHERE guild_id = ?")
        .bind(&lang)
        .bind(&guild_id)
        .execute(db)
        .await?;

    sqlx::query("UPDATE guild_settings SET log_channel_id = ? WHERE guild_id = ?")
        .bind(log_channel_id.as_deref())
        .bind(&guild_id)
        .execute(db)
        .await?;

    ctx.data().cache.set(
    ctx.guild_id().unwrap(),
    GuildSettings {
        language: lang.clone(),
            log_channel_id: log_channel_id.as_deref().and_then(|s| s.parse::<u64>().ok()),
        },
    );

    let log_display = log_channel_id
        .as_deref()
        .map(|id| format!("<#{}>", id))
        .unwrap_or_else(|| t.get(&lang, "setup.not_set"));

    let summary = serenity::CreateEmbed::new()
        .title(t.get(&lang, "setup.summary.title"))
        .color(0x2ECC71)
        .field(
            t.get(&lang, "setup.summary.language"),
            locale_display(&lang),
            true,
        )
        .field(
            t.get(&lang, "setup.summary.log_channel"),
            log_display,
            true,
        )
        .footer(serenity::CreateEmbedFooter::new(
            t.get(&lang, "setup.summary.footer"),
        ));

    reply
        .edit(
            ctx,
            poise::CreateReply::default()
                .content("")
                .embed(summary)
                .components(vec![])
                .ephemeral(true),
        )
        .await?;

    Ok(())
}