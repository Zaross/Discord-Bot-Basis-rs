use prometheus::{CounterVec, Gauge, HistogramVec, HistogramOpts, Opts, Registry};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,
    pub commands_total: CounterVec,
    pub command_errors_total: CounterVec,
    pub guild_count: Gauge,
    pub db_query_duration_seconds: HistogramVec,
    pub cache_hits: CounterVec,
    pub cache_misses: CounterVec,
    pub uptime_seconds: Gauge,
}

impl Metrics {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let registry = Registry::new();

        let commands_total = CounterVec::new(
            Opts::new("discord_commands_total", "Total slash commands executed"),
            &["command"],
        )?;
        registry.register(Box::new(commands_total.clone()))?;

        let command_errors_total = CounterVec::new(
            Opts::new("discord_command_errors_total", "Total command errors"),
            &["command"],
        )?;
        registry.register(Box::new(command_errors_total.clone()))?;

        let guild_count = Gauge::with_opts(Opts::new(
            "discord_guild_count",
            "Number of guilds the bot is in",
        ))?;
        registry.register(Box::new(guild_count.clone()))?;

        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "discord_db_query_duration_seconds",
                "Database query latency",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
            &["query"],
        )?;
        registry.register(Box::new(db_query_duration_seconds.clone()))?;

        let cache_hits = CounterVec::new(
            Opts::new("discord_cache_hits_total", "Guild settings cache hits"),
            &["type"],
        )?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = CounterVec::new(
            Opts::new("discord_cache_misses_total", "Guild settings cache misses"),
            &["type"],
        )?;
        registry.register(Box::new(cache_misses.clone()))?;

        let uptime_seconds = Gauge::with_opts(Opts::new(
            "discord_uptime_seconds",
            "Bot uptime in seconds",
        ))?;
        registry.register(Box::new(uptime_seconds.clone()))?;

        Ok(Arc::new(Self {
            registry,
            commands_total,
            command_errors_total,
            guild_count,
            db_query_duration_seconds,
            cache_hits,
            cache_misses,
            uptime_seconds,
        }))
    }

    pub fn render(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buf = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buf)
            .unwrap_or_default();
        String::from_utf8(buf).unwrap_or_default()
    }
}
