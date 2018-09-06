use config::DatabaseConfig;
use diesel::PgConnection;
use r2d2::{self, Error as R2D2Error, PooledConnection};
use r2d2_diesel::ConnectionManager;

#[derive(Clone)]
pub struct DB(r2d2::Pool<ConnectionManager<PgConnection>>);

impl DB {
    pub fn new(config: &DatabaseConfig) -> Result<DB, R2D2Error> {
        let addr = match config.port {
            Some(ref port) => format!("{}:{}", config.address, port),
            None => config.address.clone(),
        };

        let url = format!(
            "postgres://{username}:{password}@{addr}/{database}",
            username = config.username,
            password = config.password,
            addr = addr,
            database = config.database
        );

        let manager = ConnectionManager::<PgConnection>::new(url);
        r2d2::Pool::builder().build(manager).map(DB)
    }

    pub fn make_connection(&self) -> Option<PooledConnection<ConnectionManager<PgConnection>>> {
        self.0.clone().get().ok()
    }
}

pub mod models;
pub mod schema;
