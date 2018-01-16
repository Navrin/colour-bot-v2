//! # Colour Bot V2.
//!
//! A reimplmentation of the colour bot in a fully type-safe language.

extern crate bigdecimal;
#[macro_use]
extern crate diesel;
extern crate failure;
extern crate num_traits;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate read_color;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serenity;
extern crate toml;
extern crate typemap;

mod emotes;
mod colours;
mod config;
mod commands;
mod actions;
mod db;
mod utils;

use std::thread;
use std::time::Duration;

use serenity::Client;
use serenity::client::EventHandler;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::{CommandError, DispatchError};

struct Handler;
impl EventHandler for Handler {}

fn create_framework() -> StandardFramework {
    let framework = StandardFramework::new();

    framework
        .configure(|cfg| {
            cfg.prefix("!testc")
                .prefixes(vec!["!c", "!colour", "!color", "!colours", "!colors"])
                .ignore_bots(true)
                .on_mention(true)
                .allow_whitespace(true)
                .case_insensitivity(true)
        })
        .help(with_embeds)
        .group("colours", |group| {
            group
                .command("get", commands::roles::get_colour)
                .command("add", commands::roles::add_colour)
                .command("remove", commands::roles::remove_colour)
                .command("generate", commands::roles::generate_colour)
        })
        .group("utils", |group| {
            group.command("info", commands::utils::info)
        })
        .after(|_ctx, msg, cmd_name, res| {
            match res {
                Ok(_) => {
                    let result = msg.react(emotes::GREEN_TICK);

                    let _ = result.map_err(|_| {
                        let _ = msg.channel_id.send_message(|msg| {
                            msg.content("Error trying to react to a message. Check persmissions for the bot!")
                        });
                    });
                }
                Err(CommandError(err)) => {
                    let _ = msg.react(emotes::RED_CROSS);

                    let _ = msg.channel_id
                        .send_message(|msg| {
                            msg.content(format!(
                                "There was an error running the last command ({}):\n\n{}",
                                cmd_name, err
                            ))
                        })
                        .map(|reply| {
                            thread::spawn(move || {
                                thread::sleep(Duration::from_secs(8));

                                let _ = reply.delete();
                            });
                        });
                }
            };

            let msg = msg.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(8));
                let _ = msg.delete();
            });
        })
        .on_dispatch_error(|_, msg, error| {
            let _ = msg.react(emotes::RED_CROSS);

            let contents = match error {
                DispatchError::TooManyArguments { max, given } => {
                    Some(format!("Expected {} arguments, got {}. Check help for examples on how to use this command.", max, given))
                }, 
                DispatchError::NotEnoughArguments { min, given } => {
                    Some(format!("Expected {} arguments, got {}. Check help for examples on how to use this command.", min, given))
                },
                DispatchError::OnlyForGuilds => {
                    Some(format!("This command only works in a guild."))
                },
                DispatchError::LackOfPermissions(_) => {
                    Some(format!("You are lacking permissions to execute this command. Verify you have the ability to edit and manipulate roles."))
                }
                _ => None
            };

            if let Some(contents) = contents {
                let _ = msg.channel_id
                    .send_message(|m| m.content(contents))
                    .map(|reply| {
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(8));
                            let _ = reply.delete();
                            let _ = msg.delete();
                        });
                    });
            }
        })
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
