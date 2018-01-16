use typemap::Key;
use r2d2::{self, Error as R2D2Error, PooledConnection};
use r2d2_diesel::ConnectionManager;
use config::DatabaseConfig;
use diesel::PgConnection;

#[derive(Clone)]
pub struct DB(r2d2::Pool<ConnectionManager<PgConnection>>);

impl Key for DB {
    type Value = DB;
}

impl DB {
    pub fn new(config: &DatabaseConfig) -> Result<DB, R2D2Error> {
        let url = format!(
            "postgres://{username}:{password}@{address}:{port}/{database}",
            username = config.username,
            password = config.password,
            address = config.address,
            port = config.port,
            database = config.database
        );

        let manager = ConnectionManager::<PgConnection>::new(url);
        r2d2::Pool::builder().build(manager).map(|cm| DB(cm))
    }

    pub fn make_connection(&self) -> Option<PooledConnection<ConnectionManager<PgConnection>>> {
        self.0.clone().get().ok()
    }
}

pub mod models;
pub mod schema;
