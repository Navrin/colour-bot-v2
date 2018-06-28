use webserver::graphql;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{self, from_value, Value};

use failure::Error;
use hyper::client::Response;
use std::error::Error as ErrorTrait;
use std::io::prelude::*;
use webserver::graphql::GenericError;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum DiscordResponse<T> {
    Ok(T),
    Error { code: i64, message: String },
    OAuthError { error: String },
}

impl<T> Into<Result<T, Error>> for DiscordResponse<T> {
    fn into(self) -> Result<T, Error> {
        match self {
            DiscordResponse::Ok(v) => Ok(v),
            DiscordResponse::Error { message, .. } => Err(format_err!("{}", message)),
            DiscordResponse::OAuthError { error } => Err(format_err!("{}", error)),
        }
    }
}

#[derive(Debug, Display)]
pub struct DiscordError(pub String);

impl ErrorTrait for DiscordError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl From<Error> for graphql::GenericError {
    fn from(err: Error) -> graphql::GenericError {
        graphql::GenericError(format!("{}", err))
    }
}

pub trait HyperResponseExt {
    fn get_response(&mut self) -> &mut Response;

    fn text(&mut self) -> Result<String, Error> {
        let response = self.get_response();

        let mut buffer = String::new();

        response.read_to_string(&mut buffer)?;

        Ok(buffer)
    }

    fn json<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let text = self.text()?;

        let val = serde_json::from_str::<DiscordResponse<T>>(&text)?;

        val.into()
    }
}

impl HyperResponseExt for Response {
    fn get_response(&mut self) -> &mut Response {
        self
    }
}
