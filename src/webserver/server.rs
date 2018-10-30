#![allow(needless_pass_by_value)]

use CONFIG;

use self::graphql::Context;
use super::graphql;

// use rocket;
// use rocket::config::Config as RocketConfig;
// use rocket::http::Method;
// use rocket::request::{FromRequest, Outcome as RequestOutcome, Request};
// use rocket::response::content;
// use rocket::response::{NamedFile, Redirect};
// use rocket::State;

// use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};

// use juniper_rocket;

/// Old rocket version. Was dropped because
/// a) performance, rocket seems to be very slow (non-async)
/// b) matching wildcard patterns seems to be an afterthought, harder to use with SPA.println!("{}", )
// type ContextOutcome = RequestOutcome<Context, ()>;
// impl<'a, 'r> FromRequest<'a, 'r> for Context {
//     type Error = ();

//     fn from_request(request: &'a Request<'r>) -> ContextOutcome {
//         use rocket::Outcome::*;

//         let auths: Vec<_> = request.headers().get("Authorization").collect();

//         let ctx = Context {
//             token: auths
//                 .get(0)
//                 .filter(|a| a.contains("Bearer "))
//                 .map(|x| x.trim_left_matches("Bearer ").to_string()),
//         };

//         Success(ctx)
//     }
// }

// #[get("/")]
// fn index() -> io::Result<NamedFile> {
//     index_catch_all(PathBuf::new())
// }

// #[get("/<all..>", rank = 3)]
// fn index_catch_all(all: PathBuf) -> io::Result<NamedFile> {
//     let _ignore = all;
//     NamedFile::open(format!("{}/index.html", CONFIG.server.static_path))
// }

// #[get("/<all..>?<query>", rank = 3)]
// fn index_catch_all_param(all: PathBuf, query: &str) -> io::Result<NamedFile> {
//     let _ignore = all;
//     NamedFile::open(format!("{}/index.html", CONFIG.server.static_path))
// }

// #[get("/graphql")]
// fn graphiql() -> content::Html<&'static str> {
//     content::Html(include_str!("./graphql/playground.html"))
// }

// #[post("/graphql", data = "<request>")]
// fn graphql_post(
//     request: juniper_rocket::GraphQLRequest,
//     schema: State<graphql::Schema>,
//     ctx: Context,
// ) -> juniper_rocket::GraphQLResponse {
//     request.execute(&schema, &ctx)
// }

// #[get("/login")]
// fn login() -> Result<Redirect, Error> {
//     let oauth_uri = api_path!("/oauth2/authorize";
//         &[
//             ("client_id", CONFIG.discord.id.clone()),
//             ("redirect_uri", CONFIG.discord.callback_uri.clone()),
//             ("response_type", "code".into()),
//             ("scope", "guilds identify".into())
//         ]
//     );

//     Ok(Redirect::to(&oauth_uri))
// }

// #[get("/favicon.ico")]
// fn favicon() -> io::Result<NamedFile> {
//     NamedFile::open(format!("{}/favicon.ico", CONFIG.server.static_path))
// }

// #[get("/static/<file..>")]
// fn files(file: PathBuf) -> io::Result<NamedFile> {
//     let static_folder = format!("{}/static", CONFIG.server.static_path);

//     let static_path = Path::new(&static_folder);

//     NamedFile::open(static_path.join(file))
// }

// pub fn create_server() {
// #[allow(unused_mut)]
// let mut config = RocketConfig::build(
//     CONFIG
//         .server
//         .env
//         .parse()
//         .expect("An incorrect value was given for the rocket staging env!"),
// );

// #[cfg(debug_assertions)]
// {
//     let certs = CONFIG.server.certs.clone().unwrap();
//     let keys = CONFIG.server.key.clone().unwrap();

//     config = config.tls(certs.as_str(), keys.as_str());
// }

// let allowed_origins = AllowedOrigins::all();

// let cors = Cors {
//     allowed_origins,
//     allowed_methods: vec![Method::Get, Method::Options, Method::Put, Method::Post]
//         .into_iter()
//         .map(From::from)
//         .collect(),
//     allowed_headers: AllowedHeaders::all(),
//     send_wildcard: true,
//     // allow_credentials: true,
//     ..Default::default()
// };

// let config = config
//     .port(CONFIG.server.port.unwrap_or(7777))
//     .finalize()
//     .unwrap();

// rocket::custom(config, CONFIG.server.logging.unwrap_or(false))
//     .manage(graphql::Schema::new(graphql::Query, graphql::Mutation))
//     .mount(
//         "/",
//         routes![
//             index,
//             index_catch_all,
//             files,
//             favicon,
//             login,
//             graphiql,
//             graphql_post,
//         ],
//     ).attach(cors)
//     .launch();
// }
use super::graphql::{create_schema, GraphQLData, GraphQLExecutor};
/// actix implementation
/// because:
/// a) way faster than rocket
/// b) lets us do wildcard paths without hacks
use actix::prelude::*;
use actix_web::{
    fs,
    http::header::{AUTHORIZATION, LOCATION},
    http::Method,
    middleware::{self, cors::Cors},
    server, App, AsyncResponder, Error as ActixError, FutureResponse, HttpMessage, HttpRequest,
    HttpResponse,
};

use futures::prelude::*;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

struct State {
    graphql: Addr<GraphQLExecutor>,
}

fn index_route(_: &HttpRequest<State>) -> Result<fs::NamedFile, ActixError> {
    Ok(fs::NamedFile::open(format!(
        "{}/index.html",
        &CONFIG.server.static_path
    ))?)
}

fn graphiql_route(_: &HttpRequest<State>) -> Result<HttpResponse, ActixError> {
    let html = include_str!("./graphql/playground.html");

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql_route(request: &HttpRequest<State>) -> FutureResponse<HttpResponse, ActixError> {
    let request = request.clone();

    request
        .json()
        .from_err()
        .and_then(move |val: GraphQLRequest| {
            let header = request.headers().get(AUTHORIZATION);

            let state = request.state();
            let executor = &state.graphql;

            let response = executor.send(GraphQLData(
                val,
                header.and_then(|it| it.to_str().ok().map(str::to_string)),
            ));

            response.from_err().and_then(|res| match res {
                Ok(user) => Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(user)),
                Err(_) => Ok(HttpResponse::InternalServerError().into()),
            })
        }).responder()
}

fn login_route(_: &HttpRequest<State>) -> Result<HttpResponse, ActixError> {
    let oauth_uri = api_path!("/oauth2/authorize";
        &[
            ("client_id", CONFIG.discord.id.clone()),
            ("redirect_uri", CONFIG.discord.callback_uri.clone()),
            ("response_type", "code".into()),
            ("scope", "guilds identify".into())
        ]
    );

    Ok(HttpResponse::Found().header(LOCATION, oauth_uri).finish())
}

pub fn create_server() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    ::env_logger::init();

    let sys = System::new("colour-bot-server");

    let schema = Arc::new(create_schema());
    let addr = SyncArbiter::start(3, move || GraphQLExecutor {
        schema: (schema.clone()),
    });

    let inactive_server = server::new(move || {
        App::with_state(State {
            graphql: addr.clone(),
        }).middleware(middleware::Logger::default())
        .default_resource(|r| r.method(Method::GET).h(index_route))
        .handler(
            "/static",
            fs::StaticFiles::new(&format!("{}/static", CONFIG.server.static_path))
                .expect("Failed to create a static server"),
        ).configure(|app| {
            Cors::for_app(app)
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .max_age(3600)
                .resource("/graphql", |r| {
                    r.method(Method::POST).h(graphql_route);

                    r.method(Method::GET).h(graphiql_route)
                }).resource("/login", |r| r.method(Method::GET).h(login_route))
                .register()
        })
    });

    let address = format!("localhost:{}", &CONFIG.server.port.unwrap_or(7777));

    let server = if cfg!(debug_assertions) {
        inactive_server.bind_ssl(&address, {
            use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

            builder
                .set_private_key_file(
                    &CONFIG
                        .server
                        .key
                        .clone()
                        .expect("No private development SSL key defined in the config!"),
                    SslFiletype::PEM,
                ).unwrap();

            builder
                .set_certificate_chain_file(
                    &CONFIG
                        .server
                        .certs
                        .clone()
                        .expect("No private development SSL certs defined in the config!"),
                ).unwrap();

            builder
        })
    } else {
        inactive_server.bind(&address)
    };

    server
        .expect("Failure binding the actix web server!")
        .shutdown_timeout(2)
        .start();

    sys.run();
}
