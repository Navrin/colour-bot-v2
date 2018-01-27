//! # Colour Bot V2.
//!
//! A reimplmentation of the colour bot in a fully type-safe language.

extern crate bigdecimal;
extern crate cairo;
#[macro_use]
extern crate diesel;
extern crate failure;
extern crate num_traits;
extern crate parking_lot;
extern crate png;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate read_color;
extern crate resvg;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serenity;
extern crate svg;
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
use serenity::framework::standard::Args;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::{CommandError, DispatchError};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::Context;

use num_traits::ToPrimitive;

const PREFIX_LIST: [&'static str; 5] = ["!c", "!colour", "!color", "!colours", "!colors"];
const HELP_CMD_NAME: &'static str = "help";

// TODO: !! UPDATE THE COLOUR LIST WHEN COLOURS CHANGE.
// WRITE A HELP MESSAGE AND HAVE IT SENT WITH THE COLOUR MESSAGE SO IT DOESN'T HAVE TO BE TRACKED:
// MEANING THAT NO COLOUR IMAGE MESSAGE NEEDS TO BE PERSISTED, AND INSTEAD THE WHOLE CHANNEL CAN BE CLEARED

struct Handler;

impl EventHandler for Handler {
    // TODO: impl channel delete check
    // TODO: impl guild join setup and permissions check

    fn message(&self, mut ctx: Context, message: Message) {
        let starts_with_prefix = PREFIX_LIST
            .iter()
            .map(|string| string.clone().to_string())
            .map(|prefix| message.content.starts_with(&prefix))
            .any(|id| id);

        if message.author.bot || starts_with_prefix {
            return;
        }

        let connection = utils::get_connection_or_panic(&ctx);

        let colour_chan = utils::get_guild_result(&message)
            .ok()
            .and_then(|guild| {
                let id = guild.read().id;
                actions::guilds::convert_guild_to_record(&id, &connection)
            })
            .and_then(|guild_record| guild_record.channel_id)
            .and_then(|id| id.to_u64());

        let channel_id = message.channel_id;
        let message_channel_id = message.channel_id.0;

        match colour_chan {
            Some(colour_channel_id) if message_channel_id == colour_channel_id => {
                let args = Args::new(&message.content, &[" ".to_string()]);

                let result = commands::roles::get_colour_exec(&mut ctx, &message, args);

                let message_clone = message.clone();

                let _ = result
                    .map(|_| {
                        let _ = message.react(emotes::GREEN_TICK);
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(2));
                            let _ = message.delete();
                        });
                    })
                    .map_err(|CommandError(m)| {
                        let _ = message_clone.react(emotes::RED_CROSS);
                        let _ = channel_id
                            .send_message(|msg| {
                                msg.content(format!("Couldn't assign a colour due to: {}", m))
                            })
                            .map(|msg| {
                                thread::spawn(move || {
                                    thread::sleep(Duration::from_secs(8));
                                    let _ = msg.delete();
                                });
                            });
                    });
            }
            Some(_) | None => {}
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!(
            "Bot is now running!\nOn {} gulids, named as {}.",
            ready.guilds.len(),
            ready.user.name
        );
    }
}

fn create_framework() -> StandardFramework {
    let framework = StandardFramework::new();

    framework
        .configure(|cfg| {
            cfg.prefixes(PREFIX_LIST.iter())
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
                .command("list", commands::lists::list_colours)
        })
        .group("channel", |group| {
            group.command("setchannel", commands::channels::set_channel)
        })
        .group("utils", |group| {
            group.command("info", commands::utils::info)
        })
        .before(|_ctx, msg, name| {
            if name == HELP_CMD_NAME {
                let msg = msg.clone();
                let _ = msg.react(emotes::RED_CROSS);

                let _ = msg.channel_id
                    .send_message(|msg| {
                        msg.content("This command does not work outside of a DM to prevent spam, please DM me instead!")
                    })
                    .map(|res| {
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(8));
                            let _ = res.delete();
                            let _ = msg.delete();
                        });
                    });

                false
            } else {
                true
            }
        })
        .after(|_ctx, msg, cmd_name, res| {
            match res {
                Ok(_) => {
                    let result = msg.react(emotes::GREEN_TICK);

                    let _ = result.map_err(|_| {
                        let _ = msg.channel_id.send_message(|msg| {
                            msg.content("Error trying to react to a message. Check permissions for the bot!")
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
