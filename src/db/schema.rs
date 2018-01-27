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
        channel_id -> Nullable<Numeric>,
        settings -> Jsonb,
    }
}

joinable!(colours -> guilds (guild_id));
