use utils;
use actions;
use colours::ParsedColour;

use std::thread;
use std::time::Duration;

use serenity::framework::standard::CreateCommand;
use serenity::model::prelude::Message;
use serenity::model::id::RoleId;
use serenity::model::permissions::Permissions;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandError};

use num_traits::cast::ToPrimitive;

pub fn get_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["getc", "getcolour", "getcolor", "colour", "color"])
        .desc("Finds a colour role from a name and assigns it to you.")
        .guild_only(true)
        .help_available(true)
        .usage("colour")
        .example("red")
        .min_args(1)
        .exec(get_colour_exec)
}

pub fn get_colour_exec(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), CommandError> {
    let conn = utils::get_connection_or_panic(ctx);
    let colour_name = args.multiple::<String>()?.join(" ");

    let guild = msg.guild()
        .ok_or(CommandError("Could not find guild. This command only works in a guild, if you are a in a PM / Group, please only use commands that do not require any roles".to_string()))?;
    let discord_guild = guild.clone();
    let discord_guild = discord_guild.write();
    let discord_guild_id = discord_guild.id;

    let guild = actions::guilds::convert_guild_to_record(&discord_guild_id, &conn)
        .or_else(|| actions::guilds::create_new_record_from_guild(&discord_guild_id, &conn).ok())
        .ok_or(CommandError("Could not find/create a guild.".to_string()))?;

    let colour = actions::colours::find_from_name(&colour_name, &guild, &conn)
        .ok_or(CommandError(format!("Could not find a name that matches {}. Make sure you've used the correct spelling, and that you are typing a valid colour name like (red), and not a hex code like (#fff)", colour_name)))?;

    let channel = msg.channel()
        .ok_or(CommandError("Channel is null".to_string()))?;
    let channel_id = channel.id();

    let colour_init_msg = channel_id.send_message(|msg| {
        let names_differ = colour.name.to_lowercase() != colour_name.to_lowercase();
        let message_contents = if names_differ {
            format!(
                "Using nearest match for {}: `{}`, applying role now...",
                colour_name, colour.name
            )
        } else {
            "Colour found, applying role now...".to_string()
        };

        msg.content(message_contents)
    })?;

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(4));
        let _ = colour_init_msg.delete();
    });

    let colour_role = actions::colours::search_role(&colour, &discord_guild).ok_or_else(|| {
        let _ = actions::colours::remove_record(&colour, &conn);
        CommandError("Role is missing from the guild. Removing role from the list so that this doesn't occur again.".to_string())
    })?;

    actions::colours::assign_colour_to_user(&msg.author, discord_guild, &colour_role, &conn)?;

    Ok(())
}

pub fn add_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["addc", "addcolour", "addcolor"])
        .desc("Adds an existing role to the colour list.")
        .required_permissions(Permissions::MANAGE_ROLES)
        .guild_only(true)
        .help_available(true)
        .usage("<@role/role name> <colour name>")
        .example("@Redbois red")
        .min_args(2)
        .exec(add_colour_exec)
}

pub fn add_colour_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(ctx);

    let guild = msg.guild()
        .ok_or(CommandError("Could not find guild. This command only works in a guild, if you are a in a PM / Group, please only use commands that do not require any roles".to_string()))?;
    let guild = guild.read();

    let guild_id = guild.id;

    let role = utils::get_or_search_role_from_arg(&guild, &mut args)?;

    let name = args.multiple::<String>()?.join(" ");

    let check = actions::colours::find_from_role_id(&role.id, &connection);

    if check.is_some() {
        return Err(CommandError("This colour already exists in the colour list. Check the spelling of the role or mention it directly.".to_string()));
    }

    let colour_record = actions::colours::convert_role_to_record_struct(name, &role, &guild_id)
        .ok_or(CommandError(
            "Fatal error while trying to convert a role its database representation.".to_string(),
        ))?;

    actions::colours::save_record_to_db(colour_record, &connection).map_err(|e| {
        CommandError(format!(
            "Fatal error while trying to save the record into the database. Reason: {}",
            e
        ))
    })?;

    actions::guilds::update_channel_message(guild, &connection, false)?;

    Ok(())
}

pub fn remove_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["removec", "rm", "rmcolour", "rmcolor"])
        .desc("Removes a colour and its role from the list. Optionally the discord role can be preserved")
        .guild_only(true)
        .required_permissions(Permissions::MANAGE_ROLES)
        .help_available(true)
        .usage("<colour> [leave the discord role and delete only the database one]")
        .example("red false")
        .min_args(1)
        .exec(remove_colour_exec)
}

pub fn remove_colour_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(&ctx);

    let guild_res = utils::get_guild_result(&msg)?;
    let guild = guild_res.read();

    let guild_record = actions::guilds::convert_guild_to_record(&guild.id, &connection)
        .ok_or(CommandError("Guild does not exist. This means that you've never created a colour or used any colour related commands before.".to_string()))?;

    let colour_name = args.single_quoted::<String>()?;
    let colour = actions::colours::find_from_name(&colour_name, &guild_record, &connection)
        .ok_or(CommandError(format!(
            "The colour {} could not be found. Check your spelling!",
            colour_name
        )))?;

    let keep_discord_role = args.single::<bool>().unwrap_or(false);

    actions::colours::remove_record(&colour, &connection).map_err(|_| {
        CommandError("Error while trying to delete the record. Aborting!".to_string())
    })?;

    actions::guilds::update_channel_message(guild, &connection, false)?;

    if !keep_discord_role {
        let role_id = RoleId(colour.id.to_u64().ok_or(CommandError(
            "Error trying to convert a colour id to discord id.".to_string(),
        ))?);

        let guild = guild_res.read();

        let roles = guild.roles.clone();

        let role = roles
            .get(&role_id)
            .ok_or(CommandError("Error trying to delete the role associated with the colour. The role was probably deleted manually!".to_string()))?;

        // << this also prevents a deadlock >>

        role.delete()?;
    }

    Ok(())
}

pub fn generate_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["generate", "quick", "make"])
        .desc("Generates a new colour without needing a role.")
        .guild_only(true)
        .help_available(true)
        .example("#ff0000 green false")
        .usage("<colour code> <colour name> [dont assign colour on creation (default false)]")
        .min_args(2)
        .exec(generate_colour_exec)
}

pub fn generate_colour_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(ctx);

    let colour = args.single::<ParsedColour>()?;
    let name = args.single_quoted::<String>()?;
    let dont_assign = args.single::<bool>().unwrap_or(false);

    let role_colour = colour.into_role_colour();

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.write();

    let guild_id = guild.id.clone();

    let new_role = guild.create_role(|role| {
        role.name(&name)
            .colour(role_colour.0 as u64)
            .mentionable(false)
    })?;

    let colour_struct = actions::colours::convert_role_to_record_struct(name, &new_role, &guild_id)
        .ok_or_else(|| {
            let _ = new_role.delete();
            CommandError(
                "Could not convert newly created role to record. Trying to remove created role."
                    .to_string(),
            )
        })?;

    actions::colours::save_record_to_db(colour_struct, &connection).map_err(|_| {
        let _ = new_role.delete();

        CommandError(
            "Could not convert newly created role to record. Trying to remove created role."
                .to_string(),
        )
    })?;

    if !dont_assign {
        actions::colours::assign_colour_to_user(&msg.author, guild, &new_role, &connection)?
    }

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    actions::guilds::update_channel_message(guild, &connection, false)?;

    Ok(())
}
