//! # Colour Bot V2.
//!
//! A reimplmentation of the colour bot in a fully type-safe language.

extern crate bigdecimal;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate num_traits;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serenity;
extern crate toml;
extern crate typemap;

mod config;
mod commands;
mod actions;
mod db;

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
        .command("info", commands::utils::info)
        .command("get", commands::assign::get_colour)
}

fn main() {
    let config = config::get_config_from_file()
        .expect("Could not find a config file. Either provide a config.toml at the root or set a env key called COLOUR_BOT_CONFIG as a path to a config.");

    let mut client = Client::new(&config.discord.token, Handler)
        .expect("Could not initiate client. Check if your token is a *VALID* bot token.");

    let pool = db::DB::new(&config.database)
        .expect("Could not create a database connection. Verify if the given database config is valid, and your database is enabled and active.");

    {
        let mut data = client.data.lock();
        data.insert::<db::DB>(pool);
    }

    client.with_framework(create_framework());

    client.start()
        .expect("Could not start the client! Check network connection, make sure the discord servers are up.");
}
