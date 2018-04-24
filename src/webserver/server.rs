use CONFIG;

use rocket;
use rocket::config::Config as RocketConfig;
use rocket::response::NamedFile;

use std::io;
use std::path::{Path, PathBuf};

#[get("/<all..>", rank = 3)]
fn index(all: PathBuf) -> io::Result<NamedFile> {
    let _ignore = all;
    NamedFile::open(format!("{}/index.html", CONFIG.server.static_path))
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
    ).port(CONFIG.server.port.unwrap_or(7777))
        .finalize()
        .unwrap();

    rocket::custom(config, CONFIG.server.logging.unwrap_or(false))
        .mount("/", routes![index, files, favicon])
        .launch();
}
