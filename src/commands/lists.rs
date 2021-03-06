use actions;
use utils;

use std::fs;

use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandError, CreateCommand};
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::Error as SerenityError;
use serenity::CACHE;

/// Sends the colour list into the user's DMs.
pub fn list_colours(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["listc", "showall", "viewall"])
        .desc("Displays a list of all the colours in an image format.")
        .help_available(true)
        .usage("")
        .example("")
        .max_args(0)
        .exec(list_colours_exec)
}

pub fn list_colours_exec(_: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic();

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    // TODO: Options and stuff, customization for the whole family!
    let guild_record =
        actions::guilds::convert_guild_to_record(guild.id, &connection).ok_or_else(|| {
            CommandError("No guild record found, you should create some colours first.".to_string())
        })?;
    let colours = actions::colours::find_all(&guild_record, &connection)
        .ok_or_else(|| CommandError("Error getting the colours for the guild.".to_string()))?;

    let colour_list_path = actions::colours::generate_colour_image(&colours, &guild)?;

    msg.author
        .create_dm_channel()?
        .send_files(vec![colour_list_path.as_str()], |msg| {
            msg.content(format!(
                "Here are the colours for the guild \"{}\".",
                guild.name
            ))
        }).and_then(|_| {
            fs::remove_file(colour_list_path).map_err(|_| {
                SerenityError::Other("Error trying to delete the leftover colour image")
            })
        })?;

    let reply = msg.channel_id.send_message(|msg| {
        msg.content("A copy of the colour list has been sent to your DMs. To keep one in the server, set up a colour channel.")
    })?;

    delay_delete!(reply; 6);

    Ok(())
}

pub fn refresh_list(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["refresh, regenerate, reload_list"])
        .desc("Refreshes the colour list in the designated channel.")
        .help_available(true)
        .usage("")
        .example("")
        .max_args(0)
        .exec(refresh_list_exec)
}

fn refresh_list_exec(_: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic();

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    let cache = CACHE.read();
    let self_id = cache.user.id.0;

    actions::guilds::update_channel_message(&guild, self_id, &connection, true)?;
    Ok(())
}
