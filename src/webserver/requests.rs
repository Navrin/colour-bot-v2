use webserver::graphql;

use serde::de::DeserializeOwned;
use serde_json;

use failure::Error;
use hyper::client::Response;
use std::io::prelude::*;

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
