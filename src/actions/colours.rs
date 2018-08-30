use actions::guilds;

use constants::commands::MAX_STRING_COMPARE_DELTA;

use edit_distance::edit_distance;

use std::usize;

use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::BelongingToDsl;

use colours::models::ParsedColour;

use db::models::Colour;
use db::models::Guild;

use db::schema::colours as colours_schema;
use db::schema::colours::dsl as c;
use db::schema::colours::table as colours_table;

use serenity::framework::standard::CommandError;
use serenity::model::guild::{Guild as DiscordGuild, Member as DiscordMember, Role as DiscordRole};
use serenity::model::id::{GuildId, RoleId};
use serenity::model::user::User as DiscordUser;
use serenity::prelude::ModelError;
use serenity::utils::Colour as DiscordColour;
use serenity::Error as SerenityError;

use bigdecimal::BigDecimal;
use num_traits::cast::{FromPrimitive, ToPrimitive};
use parking_lot::RwLockWriteGuard;

use colours::images::ColourListBuilder;
use colours::images::Name;

/// Searches the db for a colour from a name param for a guild.
pub fn find_from_name(name: &str, guild: &Guild, connection: &PgConnection) -> Option<Colour> {
    // Colour::belonging_to(guild)
    //     .filter(c::name.ilike(format!("%{}%", name)))
    //     .get_result::<Colour>(connection)
    //     .ok()
    let name = name.trim();
    let list = find_all(guild, connection)?;

    get_nearest_colour_for_name(name, &list)
}

pub fn get_nearest_colour_for_name(name: &str, colours: &Vec<Colour>) -> Option<Colour> {
    let compare_name = |other| edit_distance(&name, other);

    let (ending_distance, closest_colour) =
        colours
            .iter()
            .fold((usize::MAX, colours.get(0)?), |(distance, last), colour| {
                let new_distance = compare_name(&colour.name);
                if distance > new_distance {
                    (new_distance, colour)
                } else {
                    (distance, last)
                }
            });

    if ending_distance > MAX_STRING_COMPARE_DELTA {
        None
    } else {
        Some(closest_colour.clone())
    }
}

/// Gets all the colours related to the guild.
pub fn find_all(guild: &Guild, connection: &PgConnection) -> Option<Vec<Colour>> {
    Colour::belonging_to(guild)
        .get_results::<Colour>(connection)
        .ok()
}

/// Turns a list of colours into it's colour name and the discord role it uses.
/// `Colour { name: "Supa Pink", guild_id: <ID>, id: <ID> }` will be converted to
/// `("Supa Pink", <Discord Role, ID = <id>>)`
pub fn convert_records_to_roles_and_name<'a>(
    colours: &Vec<Colour>,
    guild: &'a DiscordGuild,
) -> Option<Vec<(String, &'a DiscordRole)>> {
    let roles = colours
        .iter()
        .filter_map(|colour| {
            let id = colour.id.to_u64()?;

            Some((colour.name.clone(), guild.roles.get(&RoleId(id))?))
        }).collect::<Vec<(String, &DiscordRole)>>();

    if roles.len() <= 0 {
        None
    } else {
        Some(roles)
    }
}

/// Converts names and roles gained from `convert_records_to_roles_and_name` into the format needed for the colour list image.
pub fn convert_roles_and_name_to_list_type(
    colours: &Vec<(String, &DiscordRole)>,
) -> Vec<(Name, DiscordColour)> {
    colours
        .iter()
        .map(|&(ref name, ref role)| (Name(name.to_string()), role.colour.clone()))
        .collect()
}

/// Finds a colour record from the role id.
pub fn find_from_role_id(id: &RoleId, connection: &PgConnection) -> Option<Colour> {
    let id = BigDecimal::from_u64(id.0)?;

    colours_table.find(id).get_result::<Colour>(connection).ok()
}

/// Finds a role from a colour record and guild.
pub fn search_role(colour: &Colour, guild: &DiscordGuild) -> Option<DiscordRole> {
    colour
        .id
        .to_u64()
        .and_then(|id| guild.roles.get(&RoleId(id)).map(|v| v.clone()))
}

/// Removes a colour from the db.
pub fn remove_record(colour: &Colour, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(colour).execute(connection)
}

pub fn remove_multiple(
    ids: Vec<BigDecimal>,
    guild: BigDecimal,
    connection: &PgConnection,
) -> QueryResult<Vec<Colour>> {
    diesel::delete(colours_table.filter(c::id.eq(diesel::dsl::any(ids))))
        .filter(c::guild_id.eq(guild))
        .get_results(connection)
}

/// Turns a role into a colour record that *can be* inserted into the database.
/// *Note:*  this doesn't actually save the role, use `save_record_to_db` for that.
pub fn convert_role_to_record_struct(
    name: String,
    role: &DiscordRole,
    guild: &GuildId,
) -> Option<Colour> {
    BigDecimal::from_u64(role.id.0)
        .and_then(|role_id| BigDecimal::from_u64(guild.0).map(|guild_id| (role_id, guild_id)))
        .map(|(id, guild_id)| Colour { name, id, guild_id })
}

/// Saves a record generated by `convert_role_to_record_struct` into the database.
pub fn save_record_to_db(colour: Colour, connection: &PgConnection) -> QueryResult<Colour> {
    guilds::check_or_create_guild(&colour.guild_id, connection).to_result()?;

    diesel::insert_into(colours_schema::table)
        .values(&colour)
        .get_result(connection)
}

/// Simple function to give a discord member a discord role
pub fn assign_role_to_user(
    member: &mut DiscordMember,
    role: &DiscordRole,
) -> Result<(), SerenityError> {
    member.add_role(role)
}

/// Gets all the roles the user has that have a representation in the database.
pub fn get_managed_roles_from_user(
    member: &DiscordMember,
    guild: &GuildId,
    connection: &PgConnection,
) -> Result<Vec<RoleId>, SerenityError> {
    let guild_record = guilds::convert_guild_to_record(&guild, connection)
        .ok_or(SerenityError::Model(ModelError::GuildNotFound))?;

    let colours_for_guild = Colour::belonging_to(&guild_record)
        .get_results::<Colour>(connection)
        .map_err(|_| SerenityError::Other("Couldn't access the database."))?;

    let colour_ids = colours_for_guild
        .iter()
        .map(|c| c.id.clone())
        .collect::<Vec<BigDecimal>>();

    Ok(member
        .roles
        .iter()
        .filter(|r| {
            BigDecimal::from_u64(r.0)
                .map(|id| colour_ids.contains(&id))
                .unwrap_or(false)
        }).map(|id| id.clone())
        .collect::<Vec<RoleId>>())
}

/// finds the discord member for a user in a guild, clears out all the colours the user might have, then assigns a colour role to it.
pub fn assign_colour_to_user(
    author: &DiscordUser,
    mut discord_guild: RwLockWriteGuard<DiscordGuild>,
    colour_role: &DiscordRole,
    conn: &PgConnection,
) -> Result<(), SerenityError> {
    let id = discord_guild.id;

    let mut user_member = guilds::convert_user_to_member_result(&author, &mut discord_guild)?;

    let old_roles = get_managed_roles_from_user(&mut user_member, &id, &conn)?;

    if old_roles.len() > 0 {
        user_member.remove_roles(old_roles.as_slice())?;
    }

    assign_role_to_user(&mut user_member, &colour_role)
}

/// generates the data for the colour image, and then returns a path to the image.
pub fn generate_colour_image(
    colours: &Vec<Colour>,
    guild: &DiscordGuild,
) -> Result<String, CommandError> {
    let roles_and_names =
        convert_records_to_roles_and_name(&colours, &guild).ok_or(CommandError(
            "Error generating list. Possible cause: No colours exist in the database.".to_string(),
        ))?;

    let colour_list_data = convert_roles_and_name_to_list_type(&roles_and_names);

    let colour_builder = ColourListBuilder::new();

    // TODO: Options and stuff, customization for the whole family!

    let id = guild.id.clone();

    let colour_list_path = colour_builder
        .create_image(colour_list_data, id.0.to_string())
        .map_err(|_e| CommandError("Failure generating colour image.".to_string()))?;

    colour_list_path
        .to_str()
        .map(str::to_string)
        .ok_or(CommandError(
            "The image path doesn't actually exist. This a bit of a f***up. Sorry.".to_string(),
        ))
}

pub struct UpdateActionParams<'a> {
    pub colour: Colour,
    pub new_colour: Option<ParsedColour<'a>>,
    pub new_name: Option<&'a str>,
    pub change_role_name: bool,
    pub guild: &'a DiscordGuild,
}

pub fn update_colour_and_role<'a>(
    UpdateActionParams {
        colour,
        new_colour,
        new_name,
        change_role_name,
        guild,
    }: UpdateActionParams<'a>,
    connection: &PgConnection,
) -> Result<Colour, CommandError> {
    let role = search_role(&colour, guild).ok_or(CommandError(
        "Couldn't find the colour in the guild!".to_string(),
    ))?;

    role.edit(|role_edit| {
        role_edit
            .name(match new_name {
                Some(name) if change_role_name => name,
                _ => &role.name,
            }).colour(if let Some(colour) = new_colour {
                colour.into_role_colour().0 as u64
            } else {
                role.colour.0 as u64
            })
    })?;

    drop(role);

    if let Some(name) = new_name {
        diesel::update(&colour)
            .set(c::name.eq(name))
            .get_result::<Colour>(connection)
            .map_err(|_| CommandError("Error saving colour to database!".to_string()))
    } else {
        Ok(colour)
    }
}
