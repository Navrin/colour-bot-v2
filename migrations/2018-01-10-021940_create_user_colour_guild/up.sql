-- Your SQL goes here
CREATE TABLE user_colour_guild_relationships (
    id        SERIAL      PRIMARY KEY        NOT NULL,
    user_id   NUMERIC(64) REFERENCES users   NOT NULL,
    guild_id  NUMERIC(64) REFERENCES guilds  NOT NULL,
    colour_id NUMERIC(64) REFERENCES colours NOT NULL
);