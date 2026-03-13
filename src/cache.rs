use dashmap::DashMap;
use poise::serenity_prelude::GuildId;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct GuildSettings {
    pub language: String,
    pub log_channel_id: Option<u64>,
}

impl Default for GuildSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            log_channel_id: None,
        }
    }
}

struct Entry {
    settings: GuildSettings,
    cached_at: Instant,
}

pub struct GuildCache {
    inner: DashMap<GuildId, Entry>,
}

impl GuildCache {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    pub fn get(&self, guild_id: GuildId) -> Option<GuildSettings> {
        let entry = self.inner.get(&guild_id)?;
        if entry.cached_at.elapsed() > TTL {
            drop(entry);
            self.inner.remove(&guild_id);
            return None;
        }
        Some(entry.settings.clone())
    }

    pub fn set(&self, guild_id: GuildId, settings: GuildSettings) {
        self.inner.insert(
            guild_id,
            Entry {
                settings,
                cached_at: Instant::now(),
            },
        );
    }

    pub fn invalidate(&self, guild_id: GuildId) {
        self.inner.remove(&guild_id);
    }

    /// Current number of cached guilds.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
