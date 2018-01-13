-- Your SQL goes here
CREATE TABLE colours (
    -- the discord snowflake.
    -- if the colour is removed, teardown the record.
    id NUMERIC(64) PRIMARY KEY,

    -- whatever name the user calls the colour, this isn't the same as the role name.
    name TEXT NOT NULL,

    guild_id NUMERIC(64) REFERENCES guilds NOT NULL
);
