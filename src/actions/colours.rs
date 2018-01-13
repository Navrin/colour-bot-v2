use std::error::Error;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::BelongingToDsl;

use db::models::Guild;
use db::models::Colour;

use db::schema::colours as colours_schema;
use db::schema::colours::dsl as c;

use serenity::model::guild::{Guild as DiscordGuild, Role as DiscordRole};
use serenity::model::user::User as DiscordUser;
use serenity::model::id::RoleId;
use serenity::Error as SerenityError;
use serenity::prelude::ModelError;

use num_traits::cast::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;

use actions::guilds as guild_actions;

pub fn find_from_name(name: String, guild: &Guild, connection: &PgConnection) -> Option<Colour> {
    <Colour as BelongingToDsl<&Guild>>::belonging_to(guild)
        .filter(c::name.ilike(format!("%{}%", name)))
        .get_result::<Colour>(connection)
        .ok()
}

pub fn search_role(colour: &Colour, guild: &DiscordGuild) -> Option<DiscordRole> {
    colour
        .id
        .to_u64()
        .and_then(|id| guild.roles.get(&RoleId(id)).map(|v| v.clone()))
}

pub fn convert_role_to_record(
    name: String,
    role: &DiscordRole,
    guild: &DiscordGuild,
) -> Option<Colour> {
    BigDecimal::from_u64(role.id.0)
        .and_then(|role_id| BigDecimal::from_u64(guild.id.0).map(|guild_id| (role_id, guild_id)))
        .map(|(id, guild_id)| Colour { name, id, guild_id })
}

pub fn insert_record(colour: Colour, connection: &PgConnection) -> Option<Colour> {
    diesel::insert_into(colours_schema::table)
        .values(&colour)
        .get_result(connection)
        .ok()
}

pub fn assign_role_to_user(
    user: &DiscordUser,
    role: &DiscordRole,
    guild: &mut DiscordGuild,
) -> Result<(), SerenityError> {
    guild
        .members
        .get_mut(&user.id)
        .ok_or(SerenityError::Model(ModelError::InvalidUser))
        .and_then(|member| member.add_role(role))
}

pub fn get_managed_roles_from_user(
    user: &DiscordUser,
    guild: &DiscordGuild,
    connection: &PgConnection,
) -> Result<Vec<RoleId>, SerenityError> {
    guild_actions::convert_guild_to_record(guild, connection)
        .ok_or(SerenityError::Model(ModelError::GuildNotFound))
        .and_then(|guild_record| {
            Colour::belonging_to(&guild_record)
                .get_results::<Colour>(connection)
                .map_err(|e| {
                    SerenityError::Other(
                        format!("Couldn't access the database due to {}.", e.description())
                            .as_str(),
                    )
                })
        })
        .and_then(|colours_for_guild| {
            let colour_ids = colours_for_guild
                .iter()
                .map(|c| c.id)
                .collect::<Vec<BigDecimal>>();

            guild
                .members
                .get(&user.id)
                .ok_or(SerenityError::Model(ModelError::InvalidUser))
                .map(|member| {
                    member
                        .roles
                        .iter()
                        .filter(|r| {
                            BigDecimal::from_u64(r.0)
                                .map(|id| colour_ids.contains(&id))
                                .unwrap_or(false)
                        })
                        .map(|id| id.clone())
                        .collect::<Vec<RoleId>>()
                })
        })
}
