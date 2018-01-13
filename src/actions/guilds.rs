use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::BelongingToDsl;

use db::models::Guild;

use db::schema::guilds::table as guilds_table;
use db::schema::guilds;
use db::schema::guilds::dsl as g;

use serenity::model::guild::{Guild as DiscordGuild, Role as DiscordRole};
use serenity::model::user::User as DiscordUser;
use serenity::model::id::RoleId;
use serenity::Error as SerenityError;
use serenity::prelude::ModelError;

use num_traits::cast::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;

pub fn convert_guild_to_record(guild: &DiscordGuild, connection: &PgConnection) -> Option<Guild> {
    BigDecimal::from_u64(guild.id.0)
        .and_then(|id| {
            guilds_table.find(id).first(connection).ok()
        })
}
