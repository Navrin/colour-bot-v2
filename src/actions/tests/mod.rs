#![allow(dead_code)]
// snowflake ids don't need to be readable
#![allow(unreadable_literal)]

use bigdecimal::BigDecimal;
use db::models::Guild as DBGuild;
use serde_json;
use serenity::model::{
    guild::Guild,
    id::{ChannelId, RoleId},
};

lazy_static! {
    pub static ref MOCK_GUILD_DATA: Guild = serde_json::from_str(include_str!("./guild_mock.json"))
        .expect("Error serializing mock data into a serenity Guild struct");
    pub static ref DB_GUILD: DBGuild = DBGuild {
        id: BigDecimal::from(482110165651554322 as u64),
        channel_id: Some(BigDecimal::from(482110165651554327 as u64)),
        settings: serde_json::Value::Null,
        legacy: Some(true),
    };
}

pub static DEFAULT_GUILD_CHANNEL: ChannelId = ChannelId(482110165651554327);
pub static EXAMPLE_ROLE_ID: RoleId = RoleId(484529706037805056);
pub static RED_COLOUR_ID: RoleId = RoleId(483501321945612319);
pub static GREEN_COLOUR_ID: RoleId = RoleId(483501363708297225);
pub static TEST_TRANSACTION_FAILURE: &str = "Failure while attempting to create a test transaction";
pub static RECORD_MISSING_FAILURE: &str = "Record was missing from the test database!";

#[macro_export]
macro_rules! do_test_transaction {
    ($thing:expr) => {{
        use diesel::Connection;
        let conn = utils::get_connection_or_panic();

        conn.test_transaction::<_, (), _>(|| {
            let conn = &*conn;

            conn.execute(include_str!("./SETUP.sql"))
                .expect(TEST_TRANSACTION_FAILURE);

            $thing(conn);

            Ok(())
        });
    }};
}

#[macro_export]
macro_rules! login {
    () => {{
        use serenity::Client;
        use Handler;
        use CONFIG;

        Client::new(&CONFIG.discord.token, Handler)
            .expect("Could not login to discord, is there no internet connection?")
    }};
}

#[macro_export]
macro_rules! update_cache {
    ($d:expr) => {{
        use parking_lot::RwLock;
        use serenity::http;
        use std::sync::Arc;

        let mut cache = ::serenity::CACHE.write();

        cache.user = http::get_current_user().expect("failure at get current user");

        cache
            .guilds
            .insert(MOCK_GUILD_DATA.id, Arc::new(RwLock::new($d)));
    }};
}

#[cfg(test)]
pub mod channel_help;
#[cfg(test)]
pub mod colours;
#[cfg(test)]
pub mod guilds;
