use CONFIG;

use super::graphql;

use rocket;
use rocket::config::Config as RocketConfig;
use rocket::response::content;
use rocket::response::{NamedFile, Redirect};
use rocket::State;

use juniper_rocket;

use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use std::io;
use std::path::{Path, PathBuf};

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
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[post("/graphql", data = "<request>")]
fn graphql_post(
    request: juniper_rocket::GraphQLRequest,
    schema: State<graphql::Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &())
}

#[get("/login")]
fn login() -> Redirect {
    let redirect = utf8_percent_encode(&CONFIG.discord.callback_uri, DEFAULT_ENCODE_SET);

    let oauth_uri = format!(
        "https://discordapp.com/api/oauth2/authorize?client_id={client_id}&redirect_uri={redirect}&response_type=code&scope=guilds%20identify", 
        client_id=CONFIG.discord.id, 
        redirect=redirect
    );

    Redirect::to(&oauth_uri)
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
    let config = RocketConfig::build(
        CONFIG
            .server
            .env
            .parse()
            .expect("An incorrect value was given for the rocket staging env!"),
    ).tls(CONFIG.server.certs.as_str(), CONFIG.server.key.as_str())
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
        )
        .launch();
}
