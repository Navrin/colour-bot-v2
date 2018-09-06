use super::common::ColourResponse;
use super::me::Me;
use actions;
use colours::ParsedColour;
use juniper::FieldResult;
use num_traits::ToPrimitive;
use serenity::model::{
    guild::Guild as SerenityGuild,
    id::{GuildId, RoleId, UserId},
};
use serenity::CACHE;
use utils;
use webserver::graphql::GenericError;

#[derive(Clone, Debug)]
pub struct Guild(SerenityGuild);

#[derive(GraphQLObject, Deserialize, Serialize, Debug)]
pub struct Role {
    id: String,
    name: String,
    colour: String,
    permissions: i32,
    hoist: bool,
}

impl Guild {
    pub fn find_from_id(id: &str, token: &str) -> FieldResult<Guild> {
        let requestee = Me::find_from_token(token)?;
        // let guild_ids =
        //     user_guilds
        //         .iter()
        //         .map(|g| g
        //             .id
        //             .parse::<u64>()
        //             .map(|i| GuildId(i))
        //         )
        //         .collect::<Result<Vec<_>, _>>()?;

        let guild_id = GuildId(id.parse::<u64>()?);
        let cache = CACHE.read();

        let guild = cache.guilds.get(&guild_id).ok_or_else(|| {
            GenericError("This guild does not exist within the bot cache.".to_string())
        })?;
        let guild = guild.read();

        if !guild
            .members
            .contains_key(&UserId(requestee.info.id.parse::<u64>()?))
        {
            Err(GenericError(
                "You cannot lookup a guild you aren't a member on!".to_string(),
            ))?
        }

        Ok(Guild(guild.clone()))
    }
}

graphql_object!(Guild: () | &self | {
    field roles() -> Vec<Role> {
        self.0.roles.values()
            .map(|role| Role {
                id: role.id.to_string(),
                name: role.name.clone(),
                colour: format!("{}", ParsedColour::from(role.colour)),
                permissions: role.permissions.bits() as i32,
                hoist: role.hoist,
            })
            .collect()
    }


    field colours() -> FieldResult<Vec<ColourResponse>> {
        let connection = utils::get_connection_or_panic();
        let guild = actions::guilds::convert_guild_to_record(self.0.id, &connection)
            .ok_or_else(|| GenericError(format!("Could not find a guild for the id {}. Check if the bot has been added to the server before and has colours assinged to it", self.0.id)))?;

        let colours = actions::colours::find_all(&guild, &connection)
            .ok_or_else(|| GenericError("Error while attemptting to get the colours for this guild.".to_string()))?;

        Ok(
            colours
                .iter()
                .filter_map(|c| 
                    c
                        .id
                        .to_u64()
                        .and_then(|id|
                            self.0.roles
                                .get(&RoleId(id)))
                        .map(|role| 
                            ColourResponse {
                                id: c.id.to_string(),
                                name: c.name.clone(),
                                colour: format!("{}", ParsedColour::from(role.colour))
                            }
                        )
                )
                .collect()
        )
    }
});
