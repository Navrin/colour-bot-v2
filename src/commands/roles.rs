use actions::{self, colours::UpdateActionParams};
// use collector::{CollectorItem, CollectorValue, CustomCollector};
use colours::ParsedColour;
use constants::commands::{roles_edit, MAX_STRING_COMPARE_DELTA};
use dropdelete::DeleteOnDrop;
use emotes;
use utils;
// use COLLECTOR;

use std::str::FromStr;
use std::thread;
use std::usize;

use serenity::client::Context;
use serenity::framework::standard::{Args, CommandError, CreateCommand};
use serenity::model::{id::RoleId, permissions::Permissions, prelude::Message};

use edit_distance::edit_distance;
use num_traits::cast::ToPrimitive;
use prettytable::Table;

/// Most basic but most important of commands, gives the user the colour they requested, or not if it doesn't exist.
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

    delay_delete!(colour_init_msg; 4);

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
    let colour = actions::colours::find_from_name(&colour_name, &guild_record, &connection).ok_or(
        CommandError(format!(
            "The colour {} could not be found. Check your spelling!",
            colour_name
        )),
    )?;

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

        role.delete()?;
    }

    Ok(())
}

pub fn generate_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["generate", "quick", "make"])
        .desc("Generates a new colour without needing a role.")
        .guild_only(true)
        .help_available(true)
        .required_permissions(Permissions::MANAGE_ROLES)
        .example("#ff0000 green")
        .usage("<colour code> [colour name (will be generated if you dont provide one)]")
        .min_args(1)
        .exec(generate_colour_exec)
}

pub fn generate_colour_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(ctx);

    let colour = args.single::<ParsedColour>()?;
    let name = args.single_quoted_n::<String>()
        .map_err(|_| CommandError("Error parsing colour name!".to_string()))
        .and_then(|name| {
            if name.len() <= 1 {
                colour
                    .find_name()
                    .ok_or(CommandError("Could not find a colour!".to_string()))
            } else {
                Ok(name)
            }
        })?;

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

    actions::colours::assign_colour_to_user(&msg.author, guild, &new_role, &connection)?;

    let guild = utils::get_guild_result(msg)?;
    let guild = guild.read();

    actions::guilds::update_channel_message(guild, &connection, false)?;

    Ok(())
}

pub fn edit_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["edit_colour", "edit_color", "change_color", "change_colour"])
        .desc("Edit a colour on the server.")
        .guild_only(true)
        .help_available(true)
        .required_permissions(Permissions::MANAGE_ROLES)
        .usage("edit <colour name (quoted if spaces)> <info | name = foo | colour = #ff0000 | role name = bar>")
        .example("edit \"dark red\" colour = #ff0000")
        .min_args(1)
        .exec(edit_colour_exec)
}

pub fn edit_colour_exec(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let connection = utils::get_connection_or_panic(ctx);
    let colour_name = args.single_quoted::<String>()?;
    let action = args.multiple::<String>()?.join(" ");

    if action.len() <= 0 {
        return Err(CommandError(
            "No arguments were given after the colour name. Please revise the help instructions for this command".to_string(),
        ));
    }

    let guild_id = msg.guild_id().ok_or(CommandError(
        "This command only works on a guild.".to_string(),
    ))?;
    let discord_guild = utils::get_guild_result(msg)?;
    let guild_record = actions::guilds::convert_guild_to_record(&guild_id, &connection)
        .ok_or(CommandError("No guild was found in the database. This means you have not created a colour on this server yet.".to_string()))?;

    let colour_list = actions::colours::find_all(&guild_record, &connection).ok_or(CommandError(
        "This guild doesn't have any colours! What are you trying to edit?".to_string(),
    ))?;

    // no currying ;(
    let compare_name = |other| edit_distance(&colour_name, other);

    let first_colour = colour_list.first().ok_or(CommandError(
        "The colour list is empty! Try making a colour first".to_string(),
    ))?;

    let (ending_distance, closest_colour) = colour_list.iter().fold(
        (usize::MAX, first_colour),
        |(distance, last), colour| {
            let new_distance = compare_name(&colour.name);
            if distance > new_distance {
                (new_distance, colour)
            } else {
                (distance, last)
            }
        },
    );

    let closest_colour = closest_colour.clone();

    if ending_distance > MAX_STRING_COMPARE_DELTA {
        return Err(CommandError(format!("No colour name close enough to {} could be found. Check if the colour exists and that you've spelt it correctly.", colour_name)));
    }

    // the distance represents how far away the compared string is to the original one.
    // lower values == similar, higher values == not as similar

    let guild_copy = utils::get_guild_result(msg)?;
    let guild = guild_copy.clone();
    let guild = guild.read();

    let guild_closest_colour =
        actions::colours::search_role(&closest_colour, &guild).ok_or(CommandError(
            "Searched colour seems to be missing from the guild. Did someone delete the colour?"
                .to_string(),
        ))?;

    let split_action = action.split("=").collect::<Vec<_>>();

    let rhs_first = split_action
        .first()
        .ok_or(CommandError(
            "No action was given to the command! Please check the help command again".to_string(),
        ))?
        .to_lowercase();
    let lhs_second = split_action.get(1).map(|s| s.trim());

    match (rhs_first.trim(), lhs_second) {
        ("info", _) => {
            let colour_code = format!("{}", ParsedColour::from(guild_closest_colour.colour));
            // cant use the table! macro because diesel steals it
            // !: if there's a trailing comma in the macro, it might lead to infinite recursion
            let info_table = Table::init(vec![
                row!["Type", "Value", "Description"],
                row!["Name", closest_colour.name, roles_edit::NAME_DESCRIPTION],
                row!["Colour", colour_code, roles_edit::COLOUR_DESCRIPTION],
                row![
                    "Role Name",
                    guild_closest_colour.name,
                    roles_edit::ROLE_NAME_DESCRIPTION
                ],
            ]);

            let self_reply = msg.channel_id
                .send_message(|m| m.content(format!("```{}```", info_table)))?;

            delay_delete!(self_reply; 15);
        }

        ("name", value @ Some(_)) => {
            actions::colours::update_colour_and_role(
                UpdateActionParams {
                    colour: closest_colour,
                    new_colour: None,
                    new_name: value,
                    change_role_name: false,
                    guild: &guild,
                },
                &connection,
            )?;

            actions::guilds::update_channel_message(guild, &connection, false)?;
        }

        ("colour", Some(unparsed_colour)) => {
            let parsed_colour = ParsedColour::from_str(unparsed_colour).map_err(|_| {
                CommandError("The colour given was not a valid hex code.".to_string())
            })?;

            actions::colours::update_colour_and_role(
                UpdateActionParams {
                    colour: closest_colour,
                    new_colour: Some(parsed_colour),
                    new_name: None,
                    change_role_name: false,
                    guild: &guild,
                },
                &connection,
            )?;

            actions::guilds::update_channel_message(guild, &connection, false)?;
        }

        ("role name", value @ Some(_)) => {
            actions::colours::update_colour_and_role(
                UpdateActionParams {
                    colour: closest_colour,
                    new_colour: None,
                    new_name: value,
                    change_role_name: true,
                    guild: &guild,
                },
                &connection,
            )?;

            actions::guilds::update_channel_message(guild, &connection, false)?;
        }

        (act, _) => {
            return Err(CommandError(format!(
                "No commands exist for the input \"{}\". Please check the help.",
                act
            )));
        }
    }

    Ok(())
}

// macro_rules! timeout_error {
//     () => {
//         CommandError("No reply was given after 15 seconds, terminating!".to_string())
//     };
// }

// pub fn cycle_colours(cmd: CreateCommand) -> CreateCommand {
//     cmd.batch_known_as(&["cycle", "allroles", "addall"])
//         .desc("Interactively edit every existing role on the server and add them to the bot")
//         .guild_only(true)
//         .required_permissions(Permissions::MANAGE_ROLES)
//         .help_available(true)
//         .usage("cycle")
//         .example("cycle")
//         .exec(cycle_colours_exec)
// }

// pub fn cycle_colours_exec(
//     ctx: &mut Context,
//     msg: &Message,
//     mut args: Args,
// ) -> Result<(), CommandError> {
//     let connection = utils::get_connection_or_panic(ctx);
//     let msg = msg.clone();

//     let guild_id = msg.guild_id().ok_or(CommandError(
//         "This command only works on a guild.".to_string(),
//     ))?;

//     let discord_guild_orig = utils::get_guild_result(&msg)?;
//     let discord_guild = discord_guild_orig.read();

//     let guild_record = actions::guilds::convert_guild_to_record(&guild_id, &connection)
//         .ok_or(CommandError("No guild was found in the database. This means you have not created a colour on this server yet.".to_string()))?;

//     let tracked = msg.channel_id.send_message(|m| {
//         m.embed(|e| {
//             e
//                 .title("Cycling roles")
//                 .description(
//                     indoc!(
//                         "This command will let you interactively cycle through all the existing roles on this server
//                         It is recommended you perform this command in a mod channel with low activity.

//                         To continue, reply with `<yes | y | continue>`"
//                     )
//                 )
//         })
//     })?;

//     let mut tracked = DeleteOnDrop::new(tracked, 10);
//     let discord_guild = discord_guild_orig.clone();

//     thread::spawn(move || {
//         let discord_guild = discord_guild.read();

//         let result: Result<(), CommandError> = do catch {
//             let custom_collector: CustomCollector = { COLLECTOR.get_custom() };
//             let reply = custom_collector
//                 .clone()
//                 .get_message_reply(&msg)
//                 .join()
//                 .unwrap()
//                 .ok_or(timeout_error!())?;
//             let reply_content = reply.content.trim().to_lowercase();

//             match reply_content.as_str() {
//                 "yes" | "y" | "continue" => (),
//                 _ => {
//                     Err(CommandError(
//                         "Sequence was manually aborted by the user!".to_string(),
//                     ))?;
//                 }
//             }

//             reply.delete()?;

//             let everyone_role = discord_guild
//             .roles
//             .get(&RoleId(discord_guild.id.0))
//             .ok_or(CommandError(
//                 "This guild seems to... not have the everyone role? This error should never happen."
//                     .to_string(),
//             ))?;

//             let roles = discord_guild
//                 .roles
//                 .values()
//                 .filter(|role| role.name != "@everyone" && !role.permissions.administrator());

//             for role in roles {
//                 tracked.edit(|m| {
//                     m.embed(|e| {
//                         e.title(format!("Editing Role: \"{}\"", role.name))
//                             .field(
//                                 format!("Colour: {}", ParsedColour::from(role.colour)),
//                                 "The colour that will be assigned (colour is on the left side).",
//                                 false,
//                             )
//                             .field(
//                                 "Permissions check",
//                                 if role.permissions == everyone_role.permissions {
//                                     format!("{} The permissions for this role are the same for the base everyone role.", emotes::GREEN_TICK)
//                                 } else {
//                                     format!("{} The permissions for this role **do not** match the base role. \nThis role may have special permissions that you might not want to give users, so double check!", emotes::RED_CROSS)
//                                 },
//                                 false,
//                             )
//                             .field(
//                                 "Action",
//                                 indoc!(
//                                     "
//                             Would you like to add this role to the list?

//                             Type (`y` or `yes`) to include this colour
//                             Type (`n` or `no`) to go to the next colour
//                             Type (`finish` or `end`) to save your progress and finish editing
//                             Type (`cancel` or `abort`) to **remove all progress** and finish editing
//                             "
//                                 ),
//                                 false,
//                             )
//                     })
//                 })?;

//                 let reply = { COLLECTOR.get_custom() }
//                     .get_message_reply(&msg)
//                     .join()
//                     .unwrap()
//                     .ok_or(timeout_error!())?;

//                 let reply_content = reply.content.trim();
//                 reply.delete()?;

//                 match reply_content.to_lowercase().as_str() {
//                     "y" | "yes" => {}
//                     "n" | "no" => {
//                         continue;
//                     }
//                     "finish" | "end" => {
//                         break;
//                     }
//                     "cancel" | "abort" => Err(CommandError(
//                         "Sequence was manually aborted by user!".to_string(),
//                     ))?,
//                     _ => Err(CommandError(
//                         "An invalid command was given, aborting!".to_string(),
//                     ))?,
//                 }
//             }

//             Ok(())
//         };

//         match result {
//             Ok(_) => (),
//             Err(e) => {
//                 println!("{:#?}", e);
//             }
//         }
//     });

//     Ok(())
// }
