use serenity::prelude::Context;
use diesel::pg::PgConnection;
use db::DB;
use r2d2::PooledConnection;
use r2d2_diesel::ConnectionManager;

use serenity::model::prelude::{Guild, Message, Role};
use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::RwLock;

use std::sync::Arc;

pub fn get_connection_or_panic(
    context: &Context,
) -> PooledConnection<ConnectionManager<PgConnection>> {
    let manager = {
        let data = context.data.lock();
        data.get::<DB>()
            .expect("Context manager missing. Fatal Error.")
            .clone()
    };

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
