use actions::guilds;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::BelongingToDsl;

use db::models::Guild;
use db::models::Colour;

use db::schema::colours::table as colours_table;
use db::schema::colours as colours_schema;
use db::schema::colours::dsl as c;

use serenity::model::guild::{Guild as DiscordGuild, Member as DiscordMember, Role as DiscordRole};
use serenity::model::user::User as DiscordUser;
use serenity::model::id::{GuildId, RoleId};
use serenity::Error as SerenityError;
use serenity::prelude::ModelError;
use serenity::framework::standard::CommandError;
use serenity::utils::Colour as DiscordColour;

use num_traits::cast::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;
use parking_lot::RwLockWriteGuard;

use colours::images::Name;

use colours::images::ColourListBuilder;

pub fn find_from_name(name: &str, guild: &Guild, connection: &PgConnection) -> Option<Colour> {
    Colour::belonging_to(guild)
        .filter(c::name.ilike(format!("%{}%", name)))
        .get_result::<Colour>(connection)
        .ok()
}

pub fn find_all(guild: &Guild, connection: &PgConnection) -> Option<Vec<Colour>> {
    Colour::belonging_to(guild)
        .get_results::<Colour>(connection)
        .ok()
}

pub fn convert_records_to_roles_and_name<'a>(
    colours: &Vec<Colour>,
    guild: &'a DiscordGuild,
) -> Option<Vec<(String, &'a DiscordRole)>> {
    let roles = colours
        .iter()
        .filter_map(|colour| {
            let id = colour.id.to_u64()?;

            Some((colour.name.clone(), guild.roles.get(&RoleId(id))?))
        })
        .collect::<Vec<(String, &DiscordRole)>>();

    if roles.len() <= 0 {
        None
    } else {
        Some(roles)
    }
}

pub fn convert_roles_and_name_to_list_type(
    colours: &Vec<(String, &DiscordRole)>,
) -> Vec<(Name, DiscordColour)> {
    colours
        .iter()
        .map(|&(ref name, ref role)| (Name(name.to_string()), role.colour.clone()))
        .collect()
}

pub fn find_from_role_id(id: &RoleId, connection: &PgConnection) -> Option<Colour> {
    let id = BigDecimal::from_u64(id.0)?;

    colours_table.find(id).get_result::<Colour>(connection).ok()
}

pub fn search_role(colour: &Colour, guild: &DiscordGuild) -> Option<DiscordRole> {
    colour
        .id
        .to_u64()
        .and_then(|id| guild.roles.get(&RoleId(id)).map(|v| v.clone()))
}

pub fn remove_record(colour: &Colour, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(colour).execute(connection)
}

pub fn convert_role_to_record_struct(
    name: String,
    role: &DiscordRole,
    guild: &GuildId,
) -> Option<Colour> {
    BigDecimal::from_u64(role.id.0)
        .and_then(|role_id| BigDecimal::from_u64(guild.0).map(|guild_id| (role_id, guild_id)))
        .map(|(id, guild_id)| Colour { name, id, guild_id })
}

pub fn save_record_to_db(colour: Colour, connection: &PgConnection) -> QueryResult<Colour> {
    guilds::check_or_create_guild(&colour.guild_id, connection).to_result()?;

    diesel::insert_into(colours_schema::table)
        .values(&colour)
        .get_result(connection)
}

pub fn assign_role_to_user(
    member: &mut DiscordMember,
    role: &DiscordRole,
) -> Result<(), SerenityError> {
    member.add_role(role)
}

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
        })
        .map(|id| id.clone())
        .collect::<Vec<RoleId>>())
}

pub fn assign_colour_to_user(
    author: &DiscordUser,
    mut discord_guild: RwLockWriteGuard<DiscordGuild>,
    colour_role: &DiscordRole,
    conn: &PgConnection,
) -> Result<(), SerenityError> {
    let id = discord_guild.id;

    let mut user_member = guilds::convert_user_to_member_result(&author, &mut discord_guild)?;

    let old_roles = get_managed_roles_from_user(&mut user_member, &id, &conn)?;

    user_member.remove_roles(old_roles.as_slice())?;
    assign_role_to_user(&mut user_member, &colour_role)
}

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
