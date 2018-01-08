//! # Colour Bot V2.
//!
//! A reimplmentation of the colour bot in a fully type-safe language.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serenity;
extern crate toml;

mod config;

use serenity::Client;
use serenity::client::EventHandler;
use serenity::framework::StandardFramework;

struct Handler;
impl EventHandler for Handler {}

fn create_framework() -> StandardFramework {
    let framework = StandardFramework::new();

    framework
        .configure(|cfg| {
            cfg.prefixes(vec!["!c ", "c!", "c.", ".c"])
                .ignore_bots(true)
                .on_mention(true)
                .case_insensitivity(true)
        })
        .command("ping", |cmd| {
            cmd.desc("Does a pong").exec(|_, msg, _| {
                msg.reply("Pong")?;
                Ok(())
            })
        })
}

fn main() {
    let config = config::get_config_from_file()
        .expect("Could not find a config file. Either prodive a config.toml at the root or set a env key called COLOUR_BOT_CONFIG as a path to a config.");

    let mut client = Client::new(&config.discord.token, Handler)
        .expect("Could not initiate client. Check if your token is a *VALID* bot token.");

    client.with_framework(create_framework());

    client.start()
        .expect("Could not start the client! Check network connection, make sure the discord servers are up.");
}
