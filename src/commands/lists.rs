use actions;
use utils;

use colours::images;

use serenity::framework::standard::{CommandError, CreateCommand};
use serenity::model::prelude::{Channel, Guild, Message};
use serenity::framework::standard::Args;
use serenity::prelude::Context;

pub fn list_colours(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["listc", "showall", "viewall"])
        .desc("Displays a list of all the colours in an image format.")
        .guild_only(true)
        .help_available(true)
        .usage("")
        .example("")
        .max_args(0)
        .exec(list_colours_exec)
}

pub fn list_colours_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(ctx);

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    let guild_id = guild.id;

    let guild_record =
        actions::guilds::convert_guild_to_record(&guild_id, &*connection).ok_or(CommandError(
            "This guild isn't saved in the database. This means a colour hasn't been created yet."
                .to_string(),
        ))?;

    let colours = actions::colours::find_all(&guild_record, &*connection)
        .ok_or("No colours were found for the database. Consider making some first.")?;
    let roles_and_names = actions::colours::convert_records_to_roles_and_name(&colours, &guild)
        .ok_or(CommandError(
            "Error generating list. Possible cause: No colours exist in the database.".to_string(),
        ))?;

    let colour_list_data = actions::colours::convert_roles_and_name_to_list_type(&roles_and_names);

    let colour_builder = images::ColourListBuilder::new();

    // TODO: Options and stuff, customization for the whole family!

    let colour_list_path = colour_builder
        .create_image(colour_list_data, guild.id.0.to_string())
        .map_err(|_e| CommandError("Failure generating colour image.".to_string()))?;

    let colour_list_path = colour_list_path.to_str().ok_or(CommandError(
        "The image path doesn't actually exist. This a bit of a f***up. Sorry.".to_string(),
    ))?;

    msg.channel_id
        .send_files(vec![colour_list_path], |msg| msg)?;

    Ok(())
}
