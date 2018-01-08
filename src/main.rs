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

struct Handler {}
impl EventHandler for Handler {}

fn main() {
    let handler = Handler {};

    let config = config::get_config()
        .expect("Could not find a config file. Either prodive a config.toml at the root or set a env key called COLOUR_BOT_CONFIG as a path to a config.");

    let client = Client::new(&config.discord.token, handler)
        .expect("Could not initiate client. Check if your token is a *VALID* bot token.");
}
