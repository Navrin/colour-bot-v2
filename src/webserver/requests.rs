use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{self, from_value, Value};

use failure::Error;

use hyper::client::Response;
use std::error::Error as ErrorTrait;
use std::io::prelude::*;

#[derive(Debug, Display)]
pub struct DiscordError(pub String);

impl ErrorTrait for DiscordError {
    fn description(&self) -> &str {
        &self.0
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

        let val = serde_json::from_str::<T>(&text)?;

        Ok(val)
    }

    fn oauth_json<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let value = self.json::<Value>()?;

        let lifetime_v = value.clone();

        let err_opt = lifetime_v.get("error").map(|err| {
            err.as_str()
                .unwrap_or("Error field doesn't seem to be a string, check model")
        });

        let parse_attempt = from_value::<T>(value);

        let result = match err_opt {
            Some(v) => Err(DiscordError(v.to_string()))?,
            None => parse_attempt?,
        };

        Ok(result)
    }
}

impl HyperResponseExt for Response {
    fn get_response(&mut self) -> &mut Response {
        self
    }
}
