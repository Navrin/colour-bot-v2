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
        legacy -> Nullable<Bool>,
    }
}

joinable!(colours -> guilds (guild_id));

allow_tables_to_appear_in_same_query!(colours, guilds,);
