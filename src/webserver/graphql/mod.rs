use CONFIG;

mod models;
use self::models::{
    me::Me,
    guild::Guild,
    common::{TokenResponse, ColourResponse, ColourDeleteResponse, ColourUpdateInput},
};
use super::requests::HyperResponseExt;
use actions;
use std::str::FromStr;
use juniper;
use bigdecimal::ToPrimitive;
use bigdecimal::FromPrimitive;
use bigdecimal::BigDecimal;
use juniper::{FieldError, FieldResult};
use serenity::model::{
    id::{GuildId, RoleId},
};
use serenity::CACHE;
use utils;

#[derive(Debug, Display, Clone)]
pub struct GenericError(pub String);

use colours::ParsedColour;

const API_VERSION: &str = "v0.0.1";

pub struct Query;
pub struct Mutation;

pub struct Context {
    pub token: Option<String>,
}

impl Context {
    fn get_token(&self) -> FieldResult<String> {
        return Ok(self
            .token
            .clone()
            .ok_or(GenericError("Missing auth token.".to_string()))?);
    }
}

impl juniper::Context for Context {}

graphql_object!(Query: Context |&self| {

    field me(&executor) -> FieldResult<Me> {
        let ctx = executor.context();

        Ok(Me::find_from_token(&ctx.get_token()?)?)
    }

    field version() -> &str {
        API_VERSION
    }

    field token(code: String) -> FieldResult<TokenResponse> {
        let client = get_client!();

        let response = client.post(&api_path!("/oauth2/token"; &[
                ("client_id", CONFIG.discord.id.clone()),
                ("client_secret", CONFIG.discord.secret.clone()),
                ("grant_type", "authorization_code".to_string()),
                ("redirect_uri", CONFIG.discord.callback_uri.clone()),
                ("code", code)
            ]))
            .send()?
            .json::<TokenResponse>()?;

        Ok(response)
    }

    field guild(&executor, id: String) -> FieldResult<Guild> {
        let ctx = executor.context();
        let token = ctx.get_token()?;

        Guild::find_from_id(id, token)
    }
});

#[derive(GraphQLInputObject)]
struct ColourCreateInput {
    pub name: Option<String>,
    pub hex: String,
    pub role_id: Option<String>,
}


graphql_object!(Mutation: Context | &self | {
    field version() -> &str {
        API_VERSION
    }


    field create_colour(&executor, guild: String, details: ColourCreateInput) -> FieldResult<ColourResponse> {
        let guild_id = guild.parse::<u64>()?;
        let ctx = executor.context();
        let token = ctx.get_token()?;
        let connection = utils::get_connection_or_panic();

        let cache = CACHE.read();        
        let guild_rw = cache.guilds.get(&GuildId(guild_id))
            .ok_or(GenericError("Guild does not exist in the bot cache. You need to invite the bot first.".to_string()))?;
        let guild = guild_rw.read();

        let requestee = Me::find_from_token(&token)?;

        let valid_permissions = 
            requestee
                .check_permissions(
                    &guild, 
                    |permissions| permissions.administrator() || permissions.manage_roles()
                )?;
    
        if !valid_permissions {
            Err(GenericError("You do not have the permissions required to perform this action!".to_string()))?
        }

        let parsed_colour = ParsedColour::from_str(&details.hex)?;

        let name = details
            .name
            .clone()
            .or_else(|| parsed_colour.find_name())
            .ok_or(GenericError(format!("No name was found for the hex: {}, provide one.", &details.hex)))?;


        let role =  details
            .role_id
            .clone()
            .ok_or(FieldError::from(GenericError("This text should not be shown (is being mapped over)".to_string())))
            .and_then(|role| role.parse::<u64>().map_err(FieldError::from))
            .and_then(|id|
                guild
                    .roles
                    .get(&RoleId(id))
                    .cloned()
                    .ok_or(FieldError::from(GenericError("Role was not found in the bot cache.".to_string())))
            ) 
            .or_else(|_| 
                guild
                    .create_role(|r| 
                        r
                            .name(&name)
                            .colour(parsed_colour.into_role_colour().0.into())
                    )
                    .map_err(FieldError::from)
            )?;


        let colour_struct = 
            actions::colours::convert_role_to_record_struct(name, &role, &guild.id)
                .ok_or(GenericError("Error converting details for colour into a DB friendly representation".to_string()))?;

        let colour_record = 
            actions::colours::save_record_to_db(colour_struct, &connection)
                .map_err(|_| GenericError("Error saving details into the database!".to_string()))?;

        let response = ColourResponse::new_from(&colour_record, &parsed_colour);

        let self_id = cache.user.id.0;

        actions::guilds::update_channel_message(guild, self_id, &connection, false)
            .map_err(|e| GenericError(format!("Failure during channel check due to: {:#?}", e)))?;

        Ok(response)        
    }

    field delete_colours(&executor, guild: String, ids: Vec<String>) -> FieldResult<Vec<ColourDeleteResponse>> {
        let ctx = executor.context();
        let token = ctx.get_token()?;
        let connection = utils::get_connection_or_panic();

        let guild_id = GuildId(guild.parse::<u64>()?);
        let cache = CACHE.read();
        let guild = cache.guilds.get(&guild_id)
            .ok_or(GenericError(format!("Guild ID ({}) does not exist in the bot cache!", guild_id)))?;
        let guild = guild.read();

        let requestee = Me::find_from_token(&token)?;

        let valid_permissions = requestee.check_permissions(
            &guild, 
            |permissions| permissions.administrator() || permissions.manage_roles()
        )?;

        if !valid_permissions {
            Err(GenericError("You do not have the permissions required to perform this action.".to_string()))?
        }
        
        let colour_ids = 
            ids
                .iter()
                .map(String::as_str)
                .map(BigDecimal::from_str)
                .collect::<Result<Vec<BigDecimal>, _>>()?;

        let guild_id_bigdec = BigDecimal::from_u64(guild_id.0)
            .ok_or(GenericError("There was an issue getting the correct ID for the guild record.".to_string()))?;

        let colours = actions::colours::remove_multiple(colour_ids, guild_id_bigdec, &connection)?;

        let self_id = cache.user.id.0;

        actions::guilds::update_channel_message(guild, self_id, &connection, false)
                .map_err(|e| GenericError(format!("Failure during channel check due to: {:#?}", e)))?;
        
        Ok(
            colours
                .iter()
                .map(|c| ColourDeleteResponse {
                    success: true,
                    id: c.id.to_string()
                })
                .collect::<Vec<_>>()
        )
    }

    field update_colour(&executor, colour_id: String, new_data: ColourUpdateInput) -> FieldResult<ColourResponse> {
        let ctx = executor.context();
        let token = ctx.get_token()?;
        let connection = utils::get_connection_or_panic();

        let requestee = Me::find_from_token(&token)?;
        let role_id = RoleId(colour_id.parse::<u64>()?);

        let colour = actions::colours::find_from_role_id(&role_id, &connection)
            .ok_or(GenericError(format!("No colour was found for the given ID ({})", colour_id)))?;
        
        let guild_id = colour.guild_id.to_u64()
            .ok_or(GenericError("Could not convert the guild_id for the colour into a u64".to_string()))?;
        
        let cache = CACHE.read();
        let guild = cache.guilds.get(&GuildId(guild_id))
            .ok_or(
                GenericError(
                    format!(
                        "
                        The guild id ({}) associated with this colour is no longer in the bot cache.
                        The bot may have been possibily kicked while offline.
                        ", 
                    guild_id)
                )
            )?;
        let guild = guild.read();

        let valid_permissions = requestee.check_permissions(
            &guild, 
            |permissions| permissions.administrator() || permissions.manage_roles()
        )?;

        if !valid_permissions {
            Err(GenericError("You do not have the required permissions to perform this command!".to_string()))?
        }

        let new_colour = new_data.hex.and_then(|h| ParsedColour::from_str(&h).ok());
        let new_name = new_data.name.as_ref().map(String::as_str);

        let params = actions::colours::UpdateActionParams {
            colour,
            new_colour,
            new_name,
            change_role_name: new_data.update_role_name,
            guild: &guild.clone(),
        };

        let colour = actions::colours::update_colour_and_role(params, &connection)
            .map_err(|e| GenericError(format!("There was an error trying to update the colour due to {:#?}!", e)))?;
        
        let role_id = RoleId(colour.id.to_u64().ok_or(GenericError("Failure converting id into u64".to_string()))?);
        let all_roles = guild.roles.clone();
        let role = all_roles.get(&role_id)
            .ok_or(GenericError("Could not find the role for the given role_id on the colour!".to_string()))?;


        let self_id = cache.user.id.0;
        actions::guilds::update_channel_message(guild, self_id, &connection, false)
                .map_err(|e| GenericError(format!("Failure during channel check due to: {:#?}", e)))?;

        Ok(ColourResponse {
            name: colour.name,
            id: colour.id.to_string(),
            colour: format!("{}", ParsedColour::from(role.colour)),
        })
    }
});

pub type Schema = juniper::RootNode<'static, Query, Mutation>;
