use diesel::pg::PgConnection;
use r2d2::PooledConnection;
use r2d2_diesel::ConnectionManager;
use serenity::prelude::Context;
use DB;

use serenity::framework::standard::{Args, CommandError};
use serenity::model::prelude::{Guild, Message, Role};
use serenity::prelude::RwLock;

use std::sync::Arc;

pub fn get_connection_or_panic() -> PooledConnection<ConnectionManager<PgConnection>> {
    let manager = DB.clone();

    manager
        .make_connection()
        .expect("Critial failure gaining a connection")
}

pub fn get_or_search_role_from_arg(guild: &Guild, args: &mut Args) -> Result<Role, CommandError> {
    let role = args.single_quoted::<String>()?;
    role.parse::<Role>()
        .ok()
        .or_else(|| {
            guild.role_by_name(&role).map(Role::clone)
            // let mut roles = guild.roles.values();
            // roles
            //     .find(|val| val.name.contains(&role))
            //     .map(|c| c.clone())
        })
        .ok_or(CommandError(format!(
            "Cannot find a role that matches {}. Check spelling, or mention the role directly.",
            role
        )))
}

pub fn get_guild_result(msg: &Message) -> Result<Arc<RwLock<Guild>>, CommandError> {
    msg.guild()
        .ok_or(CommandError("Could not find guild. This command only works in a guild, if you are a in a PM / Group, please only use commands that do not require any roles".to_string()))
}

use typemap::Key;

/// sick shorthand for using a context item and then closing it afterwards.
/// TODO: fix weird type imference problem with the <item, (), _>
pub trait Contextable {
    fn with_item<T, E, F>(&self, cb: F) -> Result<(), E>
    where
        T: Key,
        F: Fn(&mut T::Value) -> Result<(), E>,
        T::Value: Send + Sync;
}

impl Contextable for Context {
    fn with_item<T, E, F>(&self, cb: F) -> Result<(), E>
    where
        T: Key,
        F: Fn(&mut T::Value) -> Result<(), E>,
        T::Value: Send + Sync,
    {
        let mut data = self.data.lock();
        let item = data.get_mut::<T>()
            .expect("Failure. Data missing from typemap.");

        cb(item)
    }
}
