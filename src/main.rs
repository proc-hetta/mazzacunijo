use std::{net::Ipv4Addr, sync::Arc};

use anyhow::Context;
use serde::Deserialize;
use serde_with::{DisplayFromStr, json::JsonString, serde_as};
use teloxide::prelude::*;
use teloxide::types::{Chat, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, ParseMode};
use teloxide::utils::command::BotCommands;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;
use wol::MacAddr;

const BANNER: &[u8] = include_bytes!("../assets/banner.jpg");
const CAPTION: &str = "<b>Wake-on-LAN</b>\nChoose a machine to power on.";

#[serde_as]
#[derive(Deserialize)]
struct Config {
    bot_token: String,
    allowed_ids: Vec<UserId>,
    ssh_host: String,
    #[serde_as(as = "JsonString")]
    machines: Vec<Machine>,
}

#[serde_as]
#[derive(Deserialize)]
struct Machine {
    name: String,
    #[serde_as(as = "DisplayFromStr")]
    mac: MacAddr,
    ip: Ipv4Addr,
    username: String,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Open the Wake-on-LAN control panel")]
    Start,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("MAZZACUNIJO_LOG")
                .from_env_lossy(),
        )
        .init();

    let config: Arc<Config> = envy::prefixed("MAZZACUNIJO_")
        .from_env::<Config>()
        .context("failed to load required environment")?
        .into();

    info!(
        "*** {} v{} *** {} machines, {} allowed users",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        config.machines.len(),
        config.allowed_ids.len(),
    );

    let bot = Bot::new(&config.bot_token);
    bot.set_my_commands(Command::bot_commands()).await?;

    let handler = dptree::filter(is_private)
        .branch(
            dptree::filter(is_allowed)
                .branch(
                    Update::filter_message()
                        .filter_command::<Command>()
                        .endpoint(show_panel),
                )
                .branch(Update::filter_callback_query().endpoint(wake)),
        )
        .branch(Update::filter_message().endpoint(deny));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn is_private(update: Update) -> bool {
    update.chat().is_some_and(Chat::is_private)
}

#[allow(clippy::needless_pass_by_value)]
fn is_allowed(update: Update, config: Arc<Config>) -> bool {
    update
        .from()
        .is_some_and(|u| config.allowed_ids.contains(&u.id))
}

async fn show_panel(bot: Bot, msg: Message, config: Arc<Config>) -> anyhow::Result<()> {
    bot.send_photo(msg.chat.id, InputFile::memory(BANNER))
        .caption(CAPTION)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard(&config))
        .await?;
    Ok(())
}

async fn wake(bot: Bot, q: CallbackQuery, config: Arc<Config>) -> anyhow::Result<()> {
    let machine = config
        .machines
        .iter()
        .find(|m| q.data.as_ref() == Some(&m.name));
    let Some(m) = machine else {
        bot.answer_callback_query(q.id)
            .text("Unknown machine, send /start again")
            .await?;
        return Ok(());
    };

    if let Err(e) = wol::send_wol(m.mac, Some(m.ip.into()), None) {
        warn!("{} failed to wake {}: {e}", q.from.id, m.name);
        bot.answer_callback_query(q.id)
            .text(format!("Failed to wake {}: {e}", m.name))
            .await?;
        return Ok(());
    }

    info!("{} woke {} ({})", q.from.id, m.name, m.ip);
    bot.answer_callback_query(q.id)
        .text(format!("Magic packet sent to {}", m.name))
        .await?;
    let ssh = format!(
        "ssh -p220{} {}@{}",
        m.ip.octets()[3],
        m.username,
        config.ssh_host
    );
    bot.send_message(q.from.id, format!("<code>{ssh}</code>"))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

async fn deny(bot: Bot, msg: Message) -> anyhow::Result<()> {
    warn!("Denied access to {}", msg.chat.id);
    bot.send_message(
        msg.chat.id,
        format!(
            "<b>Access denied</b>\nYour Telegram ID is <code>{}</code>.",
            msg.chat.id
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

fn keyboard(config: &Config) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(config.machines.chunks(2).map(|row| {
        row.iter()
            .map(|m| InlineKeyboardButton::callback(&m.name, &m.name))
    }))
}
