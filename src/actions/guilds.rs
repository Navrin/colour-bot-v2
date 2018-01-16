use failure::Error;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use db::models::Guild;

use db::schema::guilds::table as guilds_table;
use db::schema::guilds;
use db::schema::guilds::dsl as g;

use serenity::model::guild::{Guild as DiscordGuild, Member as DiscordMember, Role as DiscordRole};
use serenity::model::user::User as DiscordUser;
use serenity::model::id::GuildId;
use serenity::model::id::RoleId;
use serenity::Error as SerenityError;
use serenity::prelude::ModelError;

use num_traits::cast::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;

pub fn convert_guild_to_record(guild: &GuildId, connection: &PgConnection) -> Option<Guild> {
    BigDecimal::from_u64(guild.0).and_then(|id| guilds_table.find(id).first(connection).ok())
}

pub fn create_new_record_from_guild(
    guild: &GuildId,
    connection: &PgConnection,
) -> Result<Guild, Error> {
    let id = BigDecimal::from_u64(guild.0).ok_or(diesel::result::Error::NotFound)?;

    let new_guild_record = Guild { id };

    Ok(diesel::insert_into(guilds_table)
        .values(&new_guild_record)
        .get_result(connection)?)
}

pub fn convert_user_to_member<'a>(
    user: &DiscordUser,
    guild: &'a mut DiscordGuild,
) -> Option<&'a mut DiscordMember> {
    guild.members.get_mut(&user.id)
}

pub fn convert_user_to_member_result<'a>(
    user: &DiscordUser,
    guild: &'a mut DiscordGuild,
) -> Result<&'a mut DiscordMember, ModelError> {
    guild
        .members
        .get_mut(&user.id)
        .ok_or(ModelError::InvalidUser)
}
