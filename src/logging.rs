use std::path::Path;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub fn setup(logs_folder: &Path) {
    let file_all = tracing_appender::rolling::daily(logs_folder, "bot.log");
    let file_err = tracing_appender::rolling::daily(logs_folder, "error.log");

    let (nb_all, guard_all) = tracing_appender::non_blocking(file_all);
    let (nb_err, guard_err) = tracing_appender::non_blocking(file_err);

    Box::leak(Box::new(guard_all));
    Box::leak(Box::new(guard_err));

    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,serenity=warn,poise=info,sqlx=warn"));

    let file_filter = EnvFilter::new("info,serenity=warn,poise=info,sqlx=warn");

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).pretty().with_filter(console_filter))
        .with(
            fmt::layer()
                .with_target(false)
                .with_writer(nb_all)
                .with_filter(file_filter),
        )
        .with(
            fmt::layer()
                .with_target(false)
                .with_writer(nb_err)
                .with_filter(LevelFilter::ERROR),
        )
        .init();
}
