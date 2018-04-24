use CONFIG;

use constants;
use hyper::header::Header;
use juniper;
use juniper::FieldResult;
use reqwest::{self, header};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{value::from_value, Value};

header! { (ContentType, "Content-Type") => [String]}

const API_VERSION: &str = "v0.0.1";

#[derive(GraphQLObject, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}

pub struct Query;
pub struct Mutation;

fn parse_oauth_discord_response<'d, T: DeserializeOwned>(value: Value) -> Result<T, String> {
    let lifetime_v = value.clone();
    println!("{:#?}", lifetime_v);

    let err_opt = lifetime_v.get("error").map(|err| {
        err.as_str()
            .unwrap_or("Error field doesn't seem to be a string, check model")
    });

    let parse_attempt = from_value::<T>(value);

    match err_opt {
        Some(v) => Err(v.to_string()),
        None => parse_attempt.map_err(|err| format!("{}", err)),
    }
}

graphql_object!(Query: () |&self| {
    field version() -> &str {
        API_VERSION
    }

    field get_token(code: String) -> FieldResult<TokenResponse> {
        let mut client = reqwest::Client::new();
        let header = ContentType("application/x-www-form-urlencoded".into());

        let mut response = client
            .post(&format!("{}/oauth2/token", constants::webserver::DISCORD_API_URL));

        let mut response = response.basic_auth(CONFIG.discord.id.as_str(), Some(CONFIG.discord.secret.as_str()))
            .header(header)
            .header(header::ContentLength(0))
            .basic_auth(CONFIG.discord.id.clone(), Some(CONFIG.discord.secret.clone()))
            .query(&[
                // ("client_id", CONFIG.discord.id.clone()),
                // ("client_secret", CONFIG.discord.secret.clone()),
                ("grant_type", "authorization_code".to_string()),
                ("code", code)
            ]);

        println!("{:#?}", response);

        let mut response = response.send()?;

        println!("{:#?}", response.text());
        
        let response = response.json::<Value>()?;

        let data = parse_oauth_discord_response::<TokenResponse>(response)?;

        Ok(data)
    }
});

graphql_object!(Mutation: () | &self | {
    field version() -> &str {
        API_VERSION
    }
});

pub type Schema = juniper::RootNode<'static, Query, Mutation>;
