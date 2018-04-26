use CONFIG;

use super::requests::HyperResponseExt;
use actions;
use bigdecimal::ToPrimitive;
use hyper::header::Authorization;
use juniper;
use juniper::FieldResult;
use serenity::model::{guild::GuildInfo,
                      id::{GuildId, RoleId},
                      prelude::{Guild, User}};
use serenity::{cache::Cache, CACHE};
use utils;

#[derive(Debug, Display)]
struct GenericError(pub String);

use colours::ParsedColour;

use db::models::Colour;

header! { (ContentType, "Content-Type") => [String] }

const API_VERSION: &str = "v0.0.1";

#[derive(GraphQLObject, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
struct ColourReponse {
    id: String,
    name: String,
    colour: String,
}

pub struct Query;
pub struct Mutation;

graphql_object!(Query: () |&self| {
    field version() -> &str {
        API_VERSION
    }

    field token(code: String) -> FieldResult<TokenResponse> {
        let client = get_client!();
        let header = ContentType("application/x-www-form-urlencoded".into());

        let response = client.post(&api_path!("/oauth2/token"; &[
                ("client_id", CONFIG.discord.id.clone()),
                ("client_secret", CONFIG.discord.secret.clone()),
                ("grant_type", "authorization_code".to_string()),
                ("redirect_uri", CONFIG.discord.callback_uri.clone()),
                ("code", code)
            ]))
            .header(header)
            .send()?
            .oauth_json::<TokenResponse>()?;

        Ok(response)
    }

    field colours(guild: String, token: String) -> FieldResult<Vec<ColourReponse>> {
        let guild_id = GuildId(guild.parse::<u64>()?);
        let guilds = {
            let client = get_client!();
            client.get(api_path!("/users/@me/guilds"))
                .header(make_auth!(token))
                .send()?
                .json::<Vec<GuildInfo>>()?
        };

        let guild_ids = guilds.iter().map(|g| g.id).collect::<Vec<_>>();

        if !guild_ids.contains(&guild_id) {
            Err(GenericError("You can't get colours for a guild you don't belong in.".into()))?
        }

        let cache = CACHE.read();

        let guild = cache.guilds.get(&guild_id)
            .ok_or(GenericError("Requested guild does not exist within the bot cache.".into()))?
            .read();


        let connection = utils::get_connection_or_panic();
        let guild_record = actions::guilds::convert_guild_to_record(&guild_id, &connection)
            .ok_or(GenericError("The requested guild exists in the cache, but not in the database. Has a colour been added to the bot?".into()))?;

        let colours = actions::colours::find_all(&guild_record, &connection).ok_or(GenericError("Error while trying to find the colour list".into()))?;

        let data = colours
            .iter()
            .map(|colour| {
                colour.id.to_u64()
                    .and_then(|id| guild.roles.get(&RoleId(id)))
                    .map(|role| ColourReponse {
                        id: role.id.to_string(),
                        name: colour.name.clone(),
                        colour: format!("{}", ParsedColour::from(role.colour))
                    })
                    .ok_or(GenericError("There was an error converting the colours".into()))
            })
            .collect::<Result<Vec<ColourReponse>, GenericError>>()?;

        Ok(data)
    }
});

graphql_object!(Mutation: () | &self | {
    field version() -> &str {
        API_VERSION
    }
});

pub type Schema = juniper::RootNode<'static, Query, Mutation>;
