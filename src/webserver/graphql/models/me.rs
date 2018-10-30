use super::common::GuildInfo;
use juniper::FieldResult;
use serenity::model::{
    guild::Guild,
    id::{GuildId, UserId},
    Permissions,
};
use serenity::CACHE;
use webserver::graphql::GenericError;
use webserver::requests::HyperResponseExt;

pub struct Me {
    token: String,
    pub info: MeInfo,
}

#[derive(GraphQLObject, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeInfo {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
}

impl Me {
    pub fn find_from_token(token: &str) -> FieldResult<Self> {
        let client = get_client!();

        let response = client
            .get(api_path!("/users/@me"))
            .header(make_auth!(token))
            .send()?
            .json::<MeInfo>()?;

        Ok(Me {
            token: token.to_string(),
            info: response,
        })
    }

    pub fn get_guilds(&self, cached_only: Option<bool>) -> FieldResult<Vec<GuildInfo>> {
        let cached_only = cached_only.unwrap_or(false);

        let client = get_client!();

        let cache = CACHE.read();
        let guilds = &cache.guilds;

        let response = client
            .get(api_path!("/users/@me/guilds"))
            .header(make_auth!(self.token))
            .send()?
            .json::<Vec<GuildInfo>>()?
            .iter()
            .cloned()
            .filter_map(|guild| {
                guild
                    .id
                    .parse::<u64>()
                    .ok()
                    .map(|id| GuildInfo {
                        cached: guilds.contains_key(&GuildId(id)),
                        ..guild
                    }).filter(|g| if cached_only { g.cached } else { true })
            }).collect::<Vec<_>>();

        Ok(response)
    }

    pub fn check_permissions<T: FnOnce(Permissions) -> bool>(
        &self,
        guild: &Guild,
        filter: T,
    ) -> FieldResult<bool> {
        let member = guild
            .members
            .get(&UserId(self.info.id.parse::<u64>()?))
            .ok_or_else(|| {
                GenericError(
                    "You cannot perform this action without being a member of the guild!"
                        .to_string(),
                )
            })?;

        let permissions = member.permissions()?;

        Ok(filter(permissions))
    }
}

graphql_object!(Me: () | &self | {
    field guilds(
        cached_only: Option<bool> as "cached_only=true will only return guilds that the bot is also a part of."
    ) -> FieldResult<Vec<GuildInfo>> {
        Ok(self.get_guilds(cached_only)?)
    }

    field info() -> MeInfo {
        self.info.clone()
    }
});
