-- Your SQL goes here

CREATE TABLE guilds (
    -- discord guild id.
    id NUMERIC(64) PRIMARY KEY,

    channel_id NUMERIC(64),

    settings JSONB NOT NULL
);