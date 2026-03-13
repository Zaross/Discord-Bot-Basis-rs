# Discord Bot (Rust / Serenity)

A Discord bot foundation built with [Serenity](https://github.com/serenity-rs/serenity) and [Poise](https://github.com/serenity-rs/poise). Uses slash commands, structured logging, and a health check endpoint.

## Features

- ✅ Slash commands with Poise
- ✅ Structured JSON + pretty console logging (`tracing`)
- ✅ Rolling log files under `Logs/`
- ✅ Health check endpoint at `http://localhost:5000/health`
- ✅ Docker & Docker Compose support
- ✅ GitHub Actions CI/CD workflow

## Included Commands

| Command    | Description                            |
|------------|----------------------------------------|
| `/ping`    | Shows bot latency                      |
| `/info`    | Shows uptime, server count, owner info |
| `/support` | Generates a support server invite link |

## Setup

### Classic (local)

1. Make sure [Rust](https://rustup.rs/) is installed.
2. Clone this repository.
3. Copy `.env.template` to `.env` and fill in the variables:
   - `TOKEN` – Your bot token from the [Discord Developer Portal](https://discord.com/developers/applications)
   - `OWNER_ID` – Your Discord user ID
   - `SUPPORT_SERVER` – (Optional) ID of your support server
4. Run the bot:
   ```bash
   cargo run --release
   ```

### Docker Compose

1. Copy `.env.template` to `.env` and fill in the variables.
2. Start the bot:
   ```bash
   docker compose up -d
   ```

## Adding a New Command

1. Create a new file in `src/commands/`, e.g. `src/commands/hello.rs`:
   ```rust
   use crate::{Context, Error};

   /// Say hello
   #[poise::command(slash_command)]
   pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
       ctx.say("Hello!").await?;
       Ok(())
   }
   ```
2. Register it in `src/commands/mod.rs`:
   ```rust
   mod hello;
   // ...
   pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
       vec![
           // ...
           hello::hello(),
       ]
   }
   ```

## Project Structure

```
src/
├── main.rs          # Bot entry point, framework setup, logging
├── health.rs        # Axum health check server
└── commands/
    ├── mod.rs       # Command registry
    ├── ping.rs      # /ping
    ├── info.rs      # /info
    └── support.rs   # /support
```
