use actions::guilds::*;
use actions::tests::*;
use bigdecimal::{BigDecimal, FromPrimitive};
use serenity::http;
use serenity::model::{channel::ChannelType, id::GuildId, user::User};
use utils;
use CONFIG;

#[test]
fn can_transform_guild_to_record() {
    do_test_transaction!(|conn| {
        let id = MOCK_GUILD_DATA.id;

        let guild = convert_guild_to_record(&id, conn);

        assert!(guild.is_some());
    })
}

#[test]
fn can_create_guild_check_for_existing() {
    do_test_transaction!(|conn| {
        let result = check_or_create_guild(&DB_GUILD.id, conn);

        match result {
            GuildCheckStatus::AlreadyExists(_) => {
                return;
            }
            GuildCheckStatus::NewlyCreated(g) => {
                assert!(
                    false,
                    "check should have been AlreadyExists but was NewlyCreated({:#?}) instead!",
                    g
                );
            }
            GuildCheckStatus::Error(e) => {
                assert!(
                    false,
                    "check should been AlreadyExists but was an error instead ({:#?})!",
                    e
                );
            }
        }
    })
}

#[test]
fn can_create_guild_check_for_newly_created() {
    do_test_transaction!(|conn| {
        let newid = BigDecimal::from(482110165651554321 as u64);

        let result = check_or_create_guild(&newid, conn);

        match result {
            GuildCheckStatus::NewlyCreated(_) => {
                assert!(true);
            }
            GuildCheckStatus::AlreadyExists(g) => {
                assert!(
                    false,
                    "Check was AlreadyExists({:#?}) when it should have been NewlyCreated",
                    g
                );
            }
            GuildCheckStatus::Error(e) => {
                assert!(false, "Check was an error: {:#?}!", e);
            }
        }
    });
}

#[test]
fn can_make_record_repr_from_id() {
    let id = MOCK_GUILD_DATA.id;

    let guild = create_new_record_from_guild(&id);

    let guild = guild.expect("Error while converting the record!");

    assert_eq!(guild.id, DB_GUILD.id, "ID was changed during conversion!");
}

#[test]
fn can_save_converted_record_to_db() {
    do_test_transaction!(|conn| {
        let id = GuildId(MOCK_GUILD_DATA.id.0 + 1);
        let guild =
            create_new_record_from_guild(&id).expect("Error converting guild id to record!");

        let saved = save_record_into_db(&guild, conn);

        assert!(
            saved.is_ok(),
            "Failure while saving record to DB {:#?}",
            saved
        );

        let saved = saved.unwrap();

        assert_eq!(saved.id, BigDecimal::from(id.0));
    })
}

#[test]
fn can_update_record_with_channel_id() {
    do_test_transaction!(|conn| {
        login!();
        let guild = MOCK_GUILD_DATA.clone();

        update_cache!(guild.clone());

        let channel = MOCK_GUILD_DATA.create_channel("test-channel", ChannelType::Text, None);
        let channel = channel.expect("Error creating a new channel for the test guild.");

        let id = channel.id;

        let result = update_channel_id(DB_GUILD.clone(), &id, conn);

        http::delete_channel(channel.id.0).unwrap();

        let result = result.expect("Error while updating the record with the channel_id");

        assert_eq!(result.channel_id, BigDecimal::from_u64(channel.id.0));
    })
}

#[test]
fn can_convert_user_to_member() {
    login!();

    let user = http::get_current_user().unwrap();
    let mut guild = MOCK_GUILD_DATA.clone();

    let result = convert_user_to_member_result(&User::from(user), &mut guild);

    assert!(result.is_ok(), "This token isn't part of the test guild!");
}

macro_rules! setup_channel {
    () => {{
        update_cache!(MOCK_GUILD_DATA.clone());
        let cache = ::serenity::CACHE.read();

        cache.guilds.get(&MOCK_GUILD_DATA.id).unwrap().clone()
    }};
}

#[test]
fn can_update_channel_message() {
    do_test_transaction!(|conn| {
        login!();
        let guild = setup_channel!();

        let result = update_channel_message(
            guild.read(),
            CONFIG.discord.id.parse().unwrap(),
            conn,
            false,
        );

        assert!(
            result.is_ok(),
            "Result was an error when it should have succeeded! {:#?}",
            result
        );
    })
}

#[test]
fn can_update_channel_message_with_failure() {
    do_test_transaction!(|conn: &::diesel::PgConnection| {
        login!();

        let guild = setup_channel!();

        conn.execute("UPDATE guilds SET channel_id = NULL WHERE id = 482110165651554322;")
            .unwrap();

        let result =
            update_channel_message(guild.read(), CONFIG.discord.id.parse().unwrap(), conn, true);

        assert!(
            result.is_err(),
            "Update succeeded despite the channel_id being null!"
        );
    })
}
