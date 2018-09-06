#![allow(needless_pass_by_value)]

use CONFIG;

use self::graphql::Context;
use super::graphql;

use rocket;
use rocket::config::Config as RocketConfig;
use rocket::request::{FromRequest, Outcome as RequestOutcome, Request};
use rocket::response::content;
use rocket::response::{NamedFile, Redirect};
use rocket::State;

use juniper_rocket;

use failure::Error;

use std::io;
use std::path::{Path, PathBuf};

type ContextOutcome = RequestOutcome<Context, ()>;
impl<'a, 'r> FromRequest<'a, 'r> for Context {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> ContextOutcome {
        use rocket::Outcome::*;

        let auths: Vec<_> = request.headers().get("Authorization").collect();

        let ctx = Context {
            token: auths
                .get(0)
                .filter(|a| a.contains("Bearer "))
                .map(|x| x.trim_left_matches("Bearer ").to_string()),
        };

        Success(ctx)
    }
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    index_catch_all(PathBuf::new())
}

#[get("/<all..>", rank = 3)]
fn index_catch_all(all: PathBuf) -> io::Result<NamedFile> {
    let _ignore = all;
    NamedFile::open(format!("{}/index.html", CONFIG.server.static_path))
}

#[get("/graphql")]
fn graphiql() -> content::Html<&'static str> {
    content::Html(include_str!("./graphql/playground.html"))
}

#[post("/graphql", data = "<request>")]
fn graphql_post(
    request: juniper_rocket::GraphQLRequest,
    schema: State<graphql::Schema>,
    ctx: Context,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &ctx)
}

#[get("/login")]
fn login() -> Result<Redirect, Error> {
    let oauth_uri = api_path!("/oauth2/authorize"; 
        &[
            ("client_id", CONFIG.discord.id.clone()), 
            ("redirect_uri", CONFIG.discord.callback_uri.clone()), 
            ("response_type", "code".into()), 
            ("scope", "guilds identify".into())
        ]
    );

    Ok(Redirect::to(&oauth_uri))
}

#[get("/favicon.ico")]
fn favicon() -> io::Result<NamedFile> {
    NamedFile::open(format!("{}/favicon.ico", CONFIG.server.static_path))
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(&CONFIG.server.static_path).join(file)).ok()
}

pub fn create_server() {
    #[allow(unused_mut)]
    let mut config = RocketConfig::build(
        CONFIG
            .server
            .env
            .parse()
            .expect("An incorrect value was given for the rocket staging env!"),
    );

    #[cfg(debug_assertions)]
    {
        let certs = CONFIG.server.certs.clone().unwrap();
        let keys = CONFIG.server.key.clone().unwrap();

        config = config.tls(certs.as_str(), keys.as_str());
    }

    let config = config
        .port(CONFIG.server.port.unwrap_or(7777))
        .finalize()
        .unwrap();

    rocket::custom(config, CONFIG.server.logging.unwrap_or(false))
        .manage(graphql::Schema::new(graphql::Query, graphql::Mutation))
        .mount(
            "/",
            routes![
                index,
                index_catch_all,
                files,
                favicon,
                login,
                graphiql,
                graphql_post,
            ],
        ).launch();
}
