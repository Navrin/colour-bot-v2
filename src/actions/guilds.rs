use failure::Error;

use diesel;
use diesel::result::Error as DieselError;
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

fn record_exists<T>(query: QueryResult<T>) -> Option<bool> {
    match query {
        Ok(_) => Some(true),
        Err(DieselError::NotFound) => Some(false),
        Err(_) => None,
    }
}

pub enum GuildCheckStatus {
    AlreadyExists(Guild),
    NewlyCreated(Guild),
    Error(DieselError),
}

impl GuildCheckStatus {
    pub fn to_result(self) -> Result<Guild, DieselError> {
        match self {
            GuildCheckStatus::AlreadyExists(g) => Ok(g),
            GuildCheckStatus::NewlyCreated(g) => Ok(g),
            GuildCheckStatus::Error(e) => Err(e),
        }
    }

    pub fn result_to_newly(result: QueryResult<Guild>) -> GuildCheckStatus {
        match result {
            Ok(v) => GuildCheckStatus::NewlyCreated(v),
            Err(e) => GuildCheckStatus::Error(e),
        }
    }
}

pub fn check_or_create_guild(id: &BigDecimal, connection: &PgConnection) -> GuildCheckStatus {
    let query = guilds_table.find(id).get_result::<Guild>(connection);

    match query {
        Ok(v) => GuildCheckStatus::AlreadyExists(v),
        Err(DieselError::NotFound) => {
            let id = id.clone();
            let res = diesel::insert_into(guilds_table)
                .values(&Guild { id })
                .get_result::<Guild>(connection);

            GuildCheckStatus::result_to_newly(res)
        }
        Err(e) => GuildCheckStatus::Error(e),
    }
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
