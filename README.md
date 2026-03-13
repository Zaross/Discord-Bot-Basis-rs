# Discord Bot (Rust / Serenity)

A Discord bot foundation built with [Serenity](https://github.com/serenity-rs/serenity) and [Poise](https://github.com/serenity-rs/poise). Uses slash commands, per-user i18n, structured logging, and a health check endpoint.

## Features

- ✅ Slash commands with Poise
- ✅ i18n — JSON locale files, automatic fallback, `{placeholder}` substitution
- ✅ SQLite **or** Postgres — switch via `DATABASE_URL`, no recompile needed
- ✅ Auto-migrations on startup (`migrations/`)
- ✅ Pretty console + rolling log files under `{BOT_NAME}/logs/`
- ✅ Health check endpoint at `http://localhost:5000/health`
- ✅ Graceful shutdown — catches `Ctrl+C` / `SIGTERM`, closes shards & DB cleanly
- ✅ Auto-sharding — let Discord decide or set a fixed count
- ✅ Docker & Docker Compose support
- ✅ GitHub Actions CI/CD → `ghcr.io`
- ✅ Dependabot for Cargo, Docker, and GitHub Actions

## Included Commands

| Command    | Description                            |
|------------|----------------------------------------|
| `/ping`    | Shows bot latency                      |
| `/info`    | Shows uptime, server count, owner info |
| `/support` | Generates a support server invite link |

## Setup

### Local

1. Install [Rust](https://rustup.rs/).
2. Clone this repository.
3. Copy `.env.template` to `.env` and fill in the variables (see [Configuration](#configuration)).
4. Run:
   ```bash
   cargo run --release
   ```

On first start the bot creates:
```
{BOT_NAME}/
├── Database/
│   └── {BOT_NAME}.db   # SQLite only
└── logs/
    └── bot.YYYY-MM-DD.log
```

### Docker Compose

1. Copy `.env.template` to `.env` and fill in the variables.
2. Start:
   ```bash
   docker compose up -d
   ```

To use **Postgres** instead of SQLite, uncomment the `db:` service in `docker-compose.yml` and set:
```env
DATABASE_URL=postgres://bot:secret@db:5432/botdb
```

## Configuration

| Variable         | Required | Default  | Description                                                  |
|------------------|----------|----------|--------------------------------------------------------------|
| `BOT_NAME`       | No       | `Bot`    | Name used as the working folder (`{BOT_NAME}/`)              |
| `TOKEN`          | **Yes**  | —        | Bot token from the [Discord Developer Portal](https://discord.com/developers/applications) |
| `OWNER_ID`       | **Yes**  | —        | Your Discord user ID                                         |
| `SUPPORT_SERVER` | No       | —        | Guild ID of your support server                              |
| `DATABASE_URL`   | No       | `sqlite` | `sqlite`, `sqlite://path/to/file.db`, or `postgres://…`      |
| `SHARD_COUNT`    | No       | `auto`   | Number of shards, or `auto` to let Discord decide            |
| `RUST_LOG`       | No       | `info`   | Log filter, e.g. `info,serenity=warn,sqlx=warn`              |

## Adding a Command

1. Create `src/commands/hello.rs`:
   ```rust
   use crate::{ctx_ext::CtxTranslate, Context, Error};

   /// Say hello
   #[poise::command(slash_command)]
   pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
       ctx.say(ctx.t("hello.greeting")).await?;
       Ok(())
   }
   ```

2. Register it in `src/commands/mod.rs`:
   ```rust
   mod hello;

   pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
       vec![
           hello::hello(),
           // ...
       ]
   }
   ```

3. Add the translation keys to `locales/en.json` (and any other locales):
   ```json
   {
     "hello": {
       "greeting": "Hello!"
     }
   }
   ```

## Adding a Language

Drop a new JSON file into `locales/` — it is loaded automatically on startup, no code changes needed:
```
locales/
├── en.json   ← default / fallback
├── de.json
└── fr.json   ← just add this file
```

Keys use dot notation and support `{placeholder}` substitution:
```json
{ "ping": { "pong": "🏓 Pong! Latency: {ms}ms" } }
```

In a command:
```rust
ctx.t("ping.pong")
ctx.tv("ping.pong", &[("ms", "42")])
```

## Adding a Database Migration

Create a new numbered SQL file in `migrations/` — it runs automatically on the next startup:
```
migrations/
├── 0001_init.sql
└── 0002_my_new_table.sql   ← add this
```

Migrations are compatible with both SQLite and Postgres. Use the pool anywhere via `ctx.data().db.pool`.

## Project Structure

```
src/
├── main.rs            # Entry point: env, folder setup, DB, sharding, shutdown
├── db.rs              # Database abstraction (SQLite / Postgres via sqlx AnyPool)
├── health.rs          # Axum health check server (GET /health)
├── i18n.rs            # Translation engine (JSON locale files)
├── ctx_ext.rs         # ctx.t() / ctx.tv() convenience trait
└── commands/
    ├── mod.rs         # Command registry — add new commands here
    ├── ping.rs        # /ping
    ├── info.rs        # /info
    └── support.rs     # /support
migrations/
└── 0001_init.sql      # Initial schema (guild_settings table)
locales/
├── en.json            # English (default fallback)
└── de.json            # German
```
