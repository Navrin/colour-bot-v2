use std::env::var;
use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub discord: DiscordConfig,
    pub server: ServerConfig,
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    /// This is the bot token. Get one from https://discordapp.com/developers/applications/me
    pub token: String,
    /// This sets the game that will display under the bot in the members list.
    pub playing_message: Option<String>,

    pub id: String,
    pub secret: String,
    pub callback_uri: String,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub address: String,
    pub port: Option<String>,
    pub database: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub static_path: String,
    pub env: String,
    pub port: Option<u16>,
    pub logging: Option<bool>,
    pub certs: Option<String>,
    pub key: Option<String>,
}

pub fn get_config_from_file() -> Result<Config, Error> {
    let path = var("COLOUR_BOT_CONFIG")
        .or(var("COLOR_BOT_CONFIG"))
        .unwrap_or("./config.toml".into());

    let contents = File::open(path).and_then(|mut f| {
        let mut contents = String::new();
        f.read_to_string(&mut contents).map(|_| contents)
    })?;

    Ok(toml::from_str(&contents)?)
}
