table! {
    colours (id) {
        id -> Numeric,
        name -> Text,
        guild_id -> Numeric,
    }
}

table! {
    guilds (id) {
        id -> Numeric,
    }
}

table! {
    user_colour_guild_relationships (id) {
        id -> Int4,
        user_id -> Numeric,
        guild_id -> Numeric,
        colour_id -> Numeric,
    }
}

table! {
    users (id) {
        id -> Numeric,
    }
}

joinable!(colours -> guilds (guild_id));
joinable!(user_colour_guild_relationships -> colours (colour_id));
joinable!(user_colour_guild_relationships -> guilds (guild_id));
joinable!(user_colour_guild_relationships -> users (user_id));

allow_tables_to_appear_in_same_query!(
    colours,
    guilds,
    user_colour_guild_relationships,
    users,
);
