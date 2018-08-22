//! # Colour Bot V2.
//!
//! A reimplmentation of the colour bot in a fully type-safe language.
#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

// FIXME: Warn/deny for this once -DIESEL- updates for this warning.
#![allow(proc_macro_derive_resolution_fallback)]

extern crate bigdecimal;
extern crate cairo;
extern crate hsl;
extern crate hyper_native_tls;
extern crate parallel_event_emitter;
extern crate percent_encoding;
extern crate serde_urlencoded;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate diesel;
extern crate crossbeam;
#[macro_use]
extern crate derive_more;
extern crate rocket;
extern crate hyper;
extern crate edit_distance;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate failure;
extern crate juniper_rocket;
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
#[macro_use]
extern crate lazy_static;
extern crate chashmap;
extern crate serenity;
extern crate svg;
extern crate toml;
extern crate typemap;

// needs to resolve before other modules
#[macro_use]
mod macros;
mod config;
mod constants;

use serenity::model::id::MessageId;

lazy_static! {
    // static ref COLLECTOR: Collector = { Collector::new() };

    pub static ref CONFIG: config::Config = config::get_config_from_file()
        .expect("Could not find a config file. Either provide a config.toml at the root or set a env key called COLOUR_BOT_CONFIG as a path to a config.");


    pub static ref DB: db::DB = {
        db::DB::new(&CONFIG.database)
            .expect("Could not create a database connection. Verify if the given database config is valid, and your database is enabled and active.")
    };

    pub static ref CLEANER: chashmap::CHashMap<MessageId, ()> = chashmap::CHashMap::new();
}

mod actions;
mod cleaner;
mod collector;
mod colours;
mod commands;
mod db;
mod dropdelete;
mod emotes;
mod utils;
mod webserver;

use cleaner::Cleaner;

use std::thread;
use std::time::Duration;

use serenity::client::EventHandler;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandError, DispatchError};
use serenity::framework::StandardFramework;
use serenity::model::channel::{Channel, Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use serenity::Client;

use num_traits::ToPrimitive;

const PREFIX_LIST: [&str; 5] = ["!c", "!colour", "!color", "!colours", "!colors"];
const HELP_CMD_NAME: &str = "help";

// TODO: !! UPDATE THE COLOUR LIST WHEN COLOURS CHANGE.
// WRITE A HELP MESSAGE AND HAVE IT SENT WITH THE COLOUR MESSAGE SO IT DOESN'T HAVE TO BE TRACKED:
// MEANING THAT NO COLOUR IMAGE MESSAGE NEEDS TO BE PERSISTED, AND INSTEAD THE WHOLE CHANNEL CAN BE CLEARED

struct Handler;

impl EventHandler for Handler {
    // TODO: impl channel delete check
    // TODO: impl guild join setup and permissions check

    fn reaction_add(&self, _: Context, react: Reaction) {
        if let Ok(user) = react.user() {
            if user.bot {
                return;
            }
        }

        // thread::spawn(move || {
        //     let mut coll = COLLECTOR.0.lock().unwrap();

        //     coll.emit_value(CollectorItem::Reaction, CollectorValue::Reaction(react))
        //         .wait()
        //         .expect("Error emitting reaction to Collector");
        // });
    }

    /// Message handler,
    /// should be managing the colour channel and the cleaning of the channel.
    fn message(&self, mut ctx: Context, message: Message) {
        let starts_with_prefix = PREFIX_LIST
            .iter()
            .map(|string| string.clone().to_string().to_lowercase())
            .map(|prefix| message.content.to_lowercase().starts_with(&prefix))
            .any(|id| id);

        if message.author.bot {
            return;
        }
        // thread::spawn(move || {
        //     println!("about to lock ");
        //     let mut coll = COLLECTOR.0.lock().unwrap();

        //     let computation = coll.emit_value(
        //         CollectorItem::Message,
        //         CollectorValue::Message(emitted_message),
        //     );
        //     computation
        //         .wait()
        //         .expect("Error emitting message to Collector");
        //     println!("done lock");
        // });

        let connection = utils::get_connection_or_panic();

        let colour_channel_inner_opt = utils::get_guild_result(&message)
            .ok()
            .and_then(|guild| {
                let id = guild.read().id;
                actions::guilds::convert_guild_to_record(&id, &connection)
            })
            .and_then(|guild_record| guild_record.channel_id)
            .and_then(|id| id.to_u64());

        let channel_id = message.channel_id;
        let channel_id_inner = message.channel_id.0;

        // not an if let because if let doesn't support conditions. for some reason :|
        match colour_channel_inner_opt {
            // check if the message is actually in the colour channel.
            Some(colour_channel_inner) if channel_id_inner == colour_channel_inner => {
                // dont parse it as a colour if it's possibly a command.
                if !starts_with_prefix {
                    // fake args object to stimulate calling a command.
                    let args = Args::new(&message.content, &[" ".to_string()]);

                    let result = commands::roles::get_colour_exec(&mut ctx, &message, args);

                    let message_clone = message.clone();

                    let _ = result
                        .map(|_| {
                            let _ = message.react(emotes::GREEN_TICK);
                            delay_delete!(message; 2);
                        })
                        .map_err(|CommandError(m)| {
                            let _ = message_clone.react(emotes::RED_CROSS);
                            let _ = channel_id
                                .send_message(|msg| {
                                    msg.content(format!("Couldn't assign a colour due to: {}", m))
                                })
                                .map(|msg| {
                                    delay_delete!(msg; 8);
                                });
                        });
                }

                // cleaner procedure, will execute 6 seconds after the message event ends.
                // ?HACKY? relies on the .before method of the framework to be called before this does
                //       ? will .before be called sync? basically a library implmentation detail, should look into it.
                // due to having no access to commands that are invoked with the right prefix but have no way
                // to check if they're legit commands or not.
                // Advantages of this sweep approach is that bot messages and other anomalies in the channels will be purged.
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(6));
                    let colour_channel = ChannelId(channel_id_inner);
                    // collect a few messages and verify if they're commands or not.
                    // otherwise they're just loiting the channel and will be PURGED
                    let messages =
                        colour_channel.messages(|m: serenity::builder::GetMessages| m.limit(5));

                    let messages = match messages {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    let self_id = serenity::utils::with_cache(|cache| cache.user.id.clone());

                    messages
                        .iter()
                        .filter(|msg| {
                            // dont purge if
                            // A) already in the hashset of (good) messages
                            // B) from (self)

                            // TODO: There's totally a better way to do this, but my brain is having a blank and I just did this
                            // fix it later I guess

                            if msg.author.id == self_id {
                                false
                            } else if !CLEANER.contains_key(&msg.id) {
                                true
                            } else {
                                false
                            }
                        })
                        .for_each(|msg| {
                            let _ = msg.delete();
                        });

                    Ok(())
                });
            }
            _ => {}
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
            cfg.prefixes(&PREFIX_LIST)
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
                .command("edit", commands::roles::edit_colour)
                .command("list", commands::lists::list_colours)
            // .command("cycle", commands::roles::cycle_colours)
        })
        .group("channel", |group| {
            group
                .command("setchannel", commands::channels::set_channel)
                .command("refreshchannel", commands::lists::refresh_list)
        })
        .group("utils", |group| {
            group.command("info", commands::utils::info)
        })
        .before(|_, msg, name| {
            // culling help messages because they can flood the chat and dont delete themselves,
            // meaning they can linger in the colour channel.
            // therefore help messages only work in the dms.
            let channel_opt = msg.channel_id.to_channel();

            if name.to_lowercase() == HELP_CMD_NAME {
                if let Ok(Channel::Private(..)) = channel_opt {
                    return true;
                };

                let msg = msg.clone();
                let _ = msg.react(emotes::RED_CROSS);

                let _ = msg
                    .channel_id
                    .send_message(|msg| {
                        msg.content("This command does not work outside of a DM to prevent spam, please DM me instead!")
                    })
                    .map(|res| {
                        delay_delete!(res, msg; 8);
                    });

                false
            } else {
                // this will prevent the message from being sweeped by the bot.
                // not that it should be in the channel after the timer...

                {
                    CLEANER.insert(msg.id, ());
                }
                true
            }
        })
        .after(|_, msg, cmd_name, res| {
            // we're done with this message, sweep it.
            {
                CLEANER.remove(&msg.id);
            }

            let _ =
                res.map(|_| {
                    let result = msg.react(emotes::GREEN_TICK);

                    let _ = result.map_err(|_| {
                        let msg = msg.channel_id.send_message(|msg| {
                            msg.content(
                            "Error trying to react to a message. Check permissions for the bot!",
                        )
                        });

                        if let Ok(msg) = msg {
                            delay_delete!(msg; 10);
                        }
                    });
                }).map_err(|CommandError(err)| {
                    let _ = msg.react(emotes::RED_CROSS);

                    let _ = msg
                        .channel_id
                        .send_message(|msg| {
                            msg.content(format!(
                                "There was an error running the last command ({}):\n\n{}",
                                cmd_name, err
                            ))
                        })
                        .map(|reply| {
                            delay_delete!(reply; 8);
                        });
                });

            // &msg doesn't last long enough to be moved into a new thread so clone it
            let msg = msg.clone();
            delay_delete!(msg; 8);
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
                let _ = msg
                    .channel_id
                    .send_message(|m| m.content(contents))
                    .map(|reply| {
                        delay_delete!(reply, msg; 8);
                    });
            }
        })
}

fn main() {
    let mut client = Client::new(&CONFIG.discord.token, Handler)
        .expect("Could not initiate client. Check if your token is a *VALID* bot token.");

    {
        let mut data = client.data.lock();
        data.insert::<Cleaner>(Cleaner::new());
    }

    client.with_framework(create_framework());

    crossbeam::scope(|scope| {
        scope.spawn(move || {
            webserver::server::create_server();
        });

        scope.spawn(move || {
            client.start()
            .expect("Could not start the client! Check network connection, make sure the discord servers are up.");
        });
    });
}
