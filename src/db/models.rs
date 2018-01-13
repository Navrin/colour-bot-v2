use bigdecimal::BigDecimal;
use db::schema::*;

#[derive(Queryable, Insertable, Identifiable)]
#[table_name="guilds"]
pub struct Guild {
    pub id: BigDecimal,
}

#[derive(Queryable, Insertable, Identifiable)]
#[table_name="users"]
pub struct User {
    pub id: BigDecimal,
}

#[derive(Identifiable, Queryable, Associations, Insertable)]
#[belongs_to(Guild)]
#[table_name="colours"]
pub struct Colour {
    pub id: BigDecimal,
    pub name: String,
    pub guild_id: BigDecimal,
}
