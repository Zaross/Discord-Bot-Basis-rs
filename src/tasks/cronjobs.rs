use std::sync::Arc;

use poise::serenity_prelude as serenity;
use sqlx::AnyPool;
use tracing::info;

use crate::i18n::Translator;

pub async fn start_all(_http: Arc<serenity::Http>,_db: AnyPool,translator: Arc<Translator>,) {
    info!("{}", translator.get("de", "cronjobs.starting"));

}