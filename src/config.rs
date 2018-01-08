use std::fs::File;
use std::env::var;
use std::io::prelude::*;

use toml;

#[derive(Deserialize)]
pub struct Config {
    pub database: Option<()>, // TODO
    pub discord: DiscordConfig,
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    /// This is the bot token. Get one from https://discordapp.com/developers/applications/me
    pub token: String,
    /// This sets the game that will display under the bot in the members list.
    pub playing_message: Option<String>,
}

pub fn get_config_from_file() -> Option<Config> {
    let path = var("COLOUR_BOT_CONFIG")
        .or(var("COLOR_BOT_CONFIG"))
        .unwrap_or("./config.toml".into());

    File::open(path)
        .and_then(|mut f| {
            let mut contents = String::new();
            f.read_to_string(&mut contents).map(|_| contents)
        })
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
}
