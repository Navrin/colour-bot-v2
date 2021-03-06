use diesel::{Identifiable, Insertable, Queryable};
use std::default::Default;

use serde_json::Map;
use serde_json::Value;

use bigdecimal::BigDecimal;
use db::schema::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildSettings {}

impl Default for GuildSettings {
    fn default() -> Self {
        GuildSettings {}
    }
}

#[derive(Queryable, Insertable, Identifiable, Debug, Clone)]
#[table_name = "guilds"]
pub struct Guild {
    pub id: BigDecimal,
    pub channel_id: Option<BigDecimal>,
    pub settings: Value,
    pub legacy: Option<bool>,
}

impl Guild {
    pub fn with_id(id: BigDecimal) -> Self {
        Guild {
            id,
            channel_id: None,
            settings: Value::Object(Map::new()),
            legacy: Some(false),
        }
    }

    pub fn settings(&self) -> GuildSettings {
        GuildSettings {}
    }
}

#[derive(Identifiable, Queryable, Associations, Insertable, Debug, Clone, PartialEq)]
#[belongs_to(Guild)]
#[table_name = "colours"]
pub struct Colour {
    pub id: BigDecimal,
    pub name: String,
    pub guild_id: BigDecimal,
}
