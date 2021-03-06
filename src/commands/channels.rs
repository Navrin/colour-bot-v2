use actions;
use utils;

use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandError, CreateCommand};
use serenity::model::permissions::Permissions;
use serenity::model::prelude::{Channel, Message};
use serenity::prelude::Context;

/// Sets the tracked discord channel, where colour requests will be made and colour images will be posted.
pub fn set_channel(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["chan", "channel", "setchan"])
        .desc("Sets the tracked channel for the guild.")
        .help_available(true)
        .usage("<#channel mention>")
        .example("#colour_request")
        .num_args(1)
        .required_permissions(Permissions::MANAGE_CHANNELS)
        .exec(set_channel_exec)
}

pub fn set_channel_exec(
    _: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic();

    let channel = args.single::<Channel>()?;
    let channel_id = channel.id();
    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    let guild_record = actions::guilds::convert_guild_to_record(guild.id, &connection)
        .or_else(|| {
            actions::guilds::create_new_record_from_guild(guild.id)
                .and_then(|record| actions::guilds::save_record_into_db(&record, &connection))
                .ok()
        }).ok_or_else(|| {
            CommandError(
                "Couldn't convert this guild into its database representation!".to_string(),
            )
        })?;

    actions::guilds::update_channel_id(guild_record, channel_id, &connection)
        .map(|_| ())
        .map_err(|e| CommandError(format!("Could not update the channel_id due to {}", e)))
}
