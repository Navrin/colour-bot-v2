use std::fs;

use failure::Error;

use actions;

use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

use db::models::{Colour, Guild};

use db::schema::guilds::dsl as g;
use db::schema::guilds::table as guilds_table;

use serenity::framework::standard::CommandError;
use serenity::model::guild::{Guild as DiscordGuild, Member as DiscordMember};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::user::User as DiscordUser;
use serenity::prelude::ModelError;
use serenity::Error as SerenityError;

use parking_lot::RwLockReadGuard;

use bigdecimal::BigDecimal;
use num_traits::cast::{FromPrimitive, ToPrimitive};

/// Turns a discord guild into a db representation.
pub fn convert_guild_to_record(guild: GuildId, connection: &PgConnection) -> Option<Guild> {
    BigDecimal::from_u64(guild.0).and_then(|id| guilds_table.find(id).first(connection).ok())
}

/// Guild status used for finding / creating new guild records.
pub enum GuildCheckStatus {
    AlreadyExists(Guild),
    NewlyCreated(Guild),
    Error(DieselError),
}

impl GuildCheckStatus {
    /// assuming having a guild and creating a new one is intended behaviour.
    pub fn into_result(self) -> Result<Guild, DieselError> {
        match self {
            GuildCheckStatus::AlreadyExists(g) => Ok(g),
            GuildCheckStatus::NewlyCreated(g) => Ok(g),
            GuildCheckStatus::Error(e) => Err(e),
        }
    }

    /// turns a query result into a `NewlyCreated`
    pub fn result_to_newly(result: QueryResult<Guild>) -> GuildCheckStatus {
        match result {
            Ok(v) => GuildCheckStatus::NewlyCreated(v),
            Err(e) => GuildCheckStatus::Error(e),
        }
    }
}

/// Creates a new guild from and ID or returns the old one.
pub fn check_or_create_guild(id: &BigDecimal, connection: &PgConnection) -> GuildCheckStatus {
    let query = guilds_table.find(id).get_result::<Guild>(connection);

    match query {
        Ok(v) => GuildCheckStatus::AlreadyExists(v),
        Err(DieselError::NotFound) => {
            let id = id.clone();
            let res = diesel::insert_into(guilds_table)
                .values(&Guild::with_id(id))
                .get_result::<Guild>(connection);

            GuildCheckStatus::result_to_newly(res)
        }
        Err(e) => GuildCheckStatus::Error(e),
    }
}

/// Converts a GuildId into a record representation.
pub fn create_new_record_from_guild(guild: GuildId) -> Result<Guild, Error> {
    let id = BigDecimal::from_u64(guild.0).ok_or(diesel::result::Error::NotFound)?;

    Ok(Guild::with_id(id))
}

pub fn save_record_into_db(record: &Guild, connection: &PgConnection) -> Result<Guild, Error> {
    Ok(diesel::insert_into(guilds_table)
        .values(record)
        .get_result(connection)?)
}

/// Finds the given guild, then changes the `channel_id` attribute
pub fn update_channel_id(
    guild: Guild,
    channel: ChannelId,
    connection: &PgConnection,
) -> Result<Guild, Error> {
    let id = BigDecimal::from_u64(channel.0).ok_or(diesel::result::Error::NotFound)?;

    Ok(diesel::update(guilds_table.find(guild.id))
        .set(g::channel_id.eq(id))
        .get_result::<Guild>(connection)?)
}

/// Converts a discord user model into a discord member
pub fn convert_user_to_member_result<'a>(
    user: &DiscordUser,
    guild: &'a mut DiscordGuild,
) -> Result<&'a mut DiscordMember, ModelError> {
    guild
        .members
        .get_mut(&user.id)
        .ok_or(ModelError::InvalidUser)
}

/// updates the help message and colour list in the colour channel.
pub fn update_channel_message(
    guild: &RwLockReadGuard<DiscordGuild>,
    self_id: u64,
    connection: &PgConnection,
    loudly_fail: bool,
) -> Result<(), CommandError> {
    let guild_record = convert_guild_to_record(guild.id, connection)
        .ok_or_else(|| CommandError("Guild does not exist in the database".to_string()))?;

    let colours = actions::colours::find_all(&guild_record, connection)
        .ok_or_else(|| CommandError("Error trying to get list of colours.".to_string()))?;

    let path = actions::colours::generate_colour_image(&colours, &guild)?;

    let channel_id_result = guild_record
        .channel_id
        .and_then(|id| id.to_u64())
        .map(ChannelId);

    if loudly_fail {
        channel_id_result.ok_or_else(|| { 
            let _ = fs::remove_file(&path);  
            CommandError("This server does not have a colour channel set! Add a channel with the `setchannel` command!".to_string()) 
        })?;
    } else if let Some(ch) = channel_id_result {
        let old_messages = ch
            .messages(|filter| filter.limit(50))?
            .iter()
            .filter(|msg| msg.author.id.0 == self_id)
            .map(|msg| msg.id)
            .collect::<Vec<MessageId>>();

        if !old_messages.is_empty() {
            for msg in old_messages {
                ch.delete_message(msg)?;
            }
        }

        ch.send_files(vec![path.as_str()], |msg| {
            let names = colours
                .iter()
                .map(|&Colour { ref name, .. }| name.clone())
                .collect::<Vec<_>>();

            let help_message = actions::channel_help::generate_help_message(&names);

            msg.content(help_message)
        }).and_then(|_| {
            fs::remove_file(&path).map_err(|_| {
                SerenityError::Other("Error trying to delete the leftover colour image")
            })
        })?;
    }

    if fs::File::open(&path).is_ok() {
        let _ = fs::remove_file(&path);
    }

    Ok(())
}
