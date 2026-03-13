use crate::cache::GuildSettings;

pub trait CtxTranslate {
    fn t(&self, key: &str) -> String;
    fn tv(&self, key: &str, vars: &[(&str, &str)]) -> String;
    #[allow(dead_code)]
    fn guild_settings(&self) -> Option<GuildSettings>;
}

impl<'a> CtxTranslate for crate::Context<'a> {
    fn guild_settings(&self) -> Option<GuildSettings> {
        let guild_id = self.guild_id()?;
        let data = self.data();
        let settings = data.cache.get(guild_id);
        if settings.is_some() {
            data.metrics.cache_hits.with_label_values(&["guild_settings"]).inc();
        } else {
            data.metrics.cache_misses.with_label_values(&["guild_settings"]).inc();
        }
        settings
    }

    fn t(&self, key: &str) -> String {
        let locale = self.effective_locale();
        self.data().translator.get(&locale, key)
    }

    fn tv(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let locale = self.effective_locale();
        self.data().translator.get_with(&locale, key, vars)
    }
}

trait EffectiveLocale {
    fn effective_locale(&self) -> String;
}

impl<'a> EffectiveLocale for crate::Context<'a> {
    fn effective_locale(&self) -> String {
        if let Some(settings) = self.guild_id()
            .and_then(|id| self.data().cache.get(id))
        {
            return settings.language;
        }
        self.locale().unwrap_or("en").to_string()
    }
}
