use actions::colours::*;
use actions::tests::*;
use bigdecimal::ToPrimitive;
use bigdecimal::{BigDecimal, FromPrimitive};
use colours::models::ParsedColour;
use db::models::Colour;
use diesel::PgConnection;
use parking_lot::RwLock;
use serenity::{
    http,
    model::{guild::Member, id::UserId, permissions::Permissions},
    Client,
};
use std::{fs, sync::Arc};
use utils;
use Handler;
use CONFIG;

macro_rules! login {
    () => {
        Client::new(&CONFIG.discord.token, Handler)
            .expect("Could not login to discord, is there no internet connection?")
    };
}

fn get_colour_from_premade_list(name: &str, conn: &PgConnection) -> Option<Colour> {
    let alls = find_all(&DB_GUILD, conn).expect("Error getting guild colours for mock guild!");

    get_nearest_colour_for_name(name, &alls)
}

#[test]
fn can_find_colour_from_name() {
    do_test_transaction!(|conn| {
        let record = find_from_name("Red", &DB_GUILD, conn).expect(RECORD_MISSING_FAILURE);

        assert_eq!(record.name, "Red");
    });
}

#[test]
fn can_find_colour_with_name() {
    do_test_transaction!(|conn| {
        let record = get_colour_from_premade_list("Red", conn).expect(RECORD_MISSING_FAILURE);

        assert_eq!(record.name, "Red");
    });
}

#[test]
fn can_find_colour_with_misspelled_name() {
    do_test_transaction!(|conn| {
        let record = get_colour_from_premade_list("Rad", conn).expect(RECORD_MISSING_FAILURE);

        assert_eq!(record.name, "Red");
    });
}

#[test]
fn can_not_find_a_colour_that_does_not_exist() {
    do_test_transaction!(|conn| {
        let record = get_colour_from_premade_list("oaiwjawoe", conn);

        assert!(
            record.is_none(),
            "Got a result for {:#?} when it should have been none!",
            record
        );
    });
}

#[test]
fn can_convert_roles() {
    do_test_transaction!(|conn| {
        let colours = find_all(&DB_GUILD, conn).expect(RECORD_MISSING_FAILURE);
        let guild = MOCK_GUILD_DATA.clone();
        let roles = convert_records_to_roles_and_name(&colours, &guild)
            .expect("Failure while getting response while converting roles");

        assert!(!roles.is_empty());

        assert_eq!(
            roles,
            vec![
                (
                    "Red".to_string(),
                    &MOCK_GUILD_DATA.clone().roles[&RED_COLOUR_ID]
                ),
                (
                    "Green".to_string(),
                    &MOCK_GUILD_DATA.clone().roles[&GREEN_COLOUR_ID]
                )
            ]
        );
    })
}

#[test]
fn can_find_role_from_id() {
    do_test_transaction!(|conn| {
        let found_role = find_from_role_id(&RED_COLOUR_ID, conn)
            .expect("Needed Colour { name: \"Red\" }, got None");

        assert_eq!(found_role.name, "Red");
    });
}

#[test]
fn can_find_none_for_none_existant_roles() {
    do_test_transaction!(|conn| {
        let no_role = find_from_role_id(&RoleId(155686899439108096), conn);

        assert!(
            no_role.is_none(),
            "Found a role for (155686899439108096) when that shouldn't exist! ({:#?})",
            no_role,
        );
    })
}

#[test]
fn can_find_role_for_colour() {
    do_test_transaction!(|_| {
        let colour = Colour {
            id: BigDecimal::from_u64(RED_COLOUR_ID.0)
                .expect("Error converting a u64 to a bigdecimal"),
            name: "Red".to_string(),
            guild_id: BigDecimal::from_u64(MOCK_GUILD_DATA.clone().id.0)
                .expect("Error converting a u64 to a bigdecimal"),
        };

        let role = search_role(&colour, &MOCK_GUILD_DATA.clone());

        assert!(
            !role.is_none(),
            "search_role failed to find a role for {:#?}",
            colour
        );

        let role = role.unwrap();

        assert_eq!(role.id, RED_COLOUR_ID);
    });
}

#[test]
fn can_not_find_role_for_non_existant_colour() {
    do_test_transaction!(|_| {
        let colour = Colour {
            id: BigDecimal::from(155686899439108096 as u64),
            name: "empty".to_string(),
            guild_id: BigDecimal::from(155686899439108096 as u64),
        };

        let role = search_role(&colour, &MOCK_GUILD_DATA.clone());

        assert!(
            role.is_none(),
            "Found a colour when nothing should have been found! found: {:#?}",
            role
        );
    })
}

#[test]
fn can_remove_record_from_db() {
    do_test_transaction!(|conn| {
        let red = find_from_role_id(&RED_COLOUR_ID, conn).unwrap();

        let affected =
            remove_record(&red, conn).expect("Record did not get removed from the database.");

        assert!(
            affected == 1,
            "Query removed records but it was less or more than 1 ({})!",
            affected
        );
    });
}

#[test]
fn can_not_remove_non_existant_record() {
    do_test_transaction!(|conn| {
        let colour = Colour {
            id: BigDecimal::from(155686899439108096 as u64),
            name: "empty".to_string(),
            guild_id: BigDecimal::from(155686899439108096 as u64),
        };

        let result = remove_record(&colour, conn).expect("error trying to unwrap result");

        assert!(
            result == 0,
            "Result should have been 0 but instead was {}",
            result,
        );
    })
}

#[test]
fn can_remove_multiple_records() {
    do_test_transaction!(|conn| {
        let all_ids = vec![
            BigDecimal::from(RED_COLOUR_ID.0),
            BigDecimal::from(GREEN_COLOUR_ID.0),
        ];

        let results = remove_multiple(
            all_ids,
            BigDecimal::from(MOCK_GUILD_DATA.clone().id.0),
            conn,
        );

        assert!(results.is_ok(), "Results are none when they should be ok!");

        let results = results.unwrap();

        assert!(
            results.len() == 2,
            "Removing multiple records return a vec of len {} when it should have been 2.",
            results.len(),
        );
    })
}

#[test]
fn can_assign_role_to_user() {
    login!();

    let red_role = {
        let mut guild = MOCK_GUILD_DATA.clone();
        guild.roles.get_mut(&RED_COLOUR_ID).unwrap().clone()
    };

    let mut guild = MOCK_GUILD_DATA.clone();

    let own_user = guild
        .members
        .get_mut(&UserId(CONFIG.discord.id.parse().unwrap()))
        .expect("This bot needs to be part of the bot testing server");

    // cleanup
    let _ = own_user.remove_role(red_role.id);

    let result = assign_role_to_user(own_user, &red_role);

    let _ = own_user.remove_role(red_role.id);

    assert!(
        result.is_ok(),
        "Expected role assignment to work, got {:?} instead!",
        result
    );
}

#[test]
fn can_convert_role_to_record() {
    let role = MOCK_GUILD_DATA
        .roles
        .get(&EXAMPLE_ROLE_ID)
        .expect("Example role is not in the mock guild data");
    let colour = convert_role_to_record_struct("colour".to_string(), role, &MOCK_GUILD_DATA.id);

    let colour = colour.expect("Error during the conversion of the role to a record struct!");

    assert_eq!(
        colour.name, "colour",
        "colour name is {} when it should have been 'colour'",
        colour.name
    );

    assert_eq!(colour.id.to_u64().unwrap(), EXAMPLE_ROLE_ID.0);
}

#[test]
fn can_save_record_to_database() {
    let colour = Colour {
        id: BigDecimal::from(EXAMPLE_ROLE_ID.0),
        guild_id: BigDecimal::from(MOCK_GUILD_DATA.id.0),
        name: "colour".to_string(),
    };

    do_test_transaction!(|conn| {
        let result = save_record_to_db(colour, conn);

        let result = result.expect("Failure while inserting record into database");

        assert_eq!(result.id.to_u64().unwrap(), EXAMPLE_ROLE_ID.0);

        let found_role = find_from_role_id(&EXAMPLE_ROLE_ID, conn);

        let found_role = found_role.expect("Could not find the inserted record in the DB!");

        assert_eq!(found_role, result);
    })
}

fn get_member() -> Member {
    let mut guild = MOCK_GUILD_DATA.clone();

    guild
        .members
        .get_mut(&UserId(CONFIG.discord.id.parse().unwrap()))
        .expect("This bot is not part of testing server.")
        .clone()
}

#[test]
fn can_get_colours_from_user() {
    do_test_transaction!(|conn| {
        login!();

        let mut member = get_member();

        let guild = MOCK_GUILD_DATA.clone();
        let red_role = guild.roles.get(&RED_COLOUR_ID).unwrap();
        let green_role = guild.roles.get(&GREEN_COLOUR_ID).unwrap();

        let roles = [red_role.id, green_role.id];

        // making sure the user is in a blank state
        let _ = member.remove_roles(&roles);

        let result = member.add_roles(&roles);
        let found_roles = get_managed_roles_from_user(&member, &guild.id.clone(), conn);

        let _ = member.remove_roles(&roles);

        result.expect("Error adding roles to the member of the guild!");

        let found_roles = found_roles
            .expect("Error while accessing database and getting managed roles for a member.");
        let roles_count = found_roles.len();

        assert!(roles_count == 2, "Found extra or not enough roles while getting managed roles! Expected len of 2 but got {}!", roles_count);
    })
}

#[test]
fn can_assign_a_colour_to_user() {
    do_test_transaction!(|conn| {
        login!();

        let member = get_member();

        let guild = MOCK_GUILD_DATA.clone();

        let colour = guild.roles.get(&RED_COLOUR_ID).unwrap();

        let guildrw = RwLock::new(guild.clone());

        let result = assign_colour_to_user(&member.user.read(), guildrw.write(), &colour, conn);
        let mut guild = guildrw.try_write().unwrap();

        let member = guild
            .members
            .get_mut(&UserId(CONFIG.discord.id.parse().unwrap()))
            .unwrap();

        let member_has = member.roles.contains(&RED_COLOUR_ID);

        let _ = member.remove_role(colour);

        result.expect("assign_colour_to_user should have been Ok, instead got:");
        assert!(
            member_has,
            "Member should have been assigned a role, instead was false!"
        );
    })
}

#[test]
fn can_reassign_colours_to_managed_users() {
    do_test_transaction!(|conn| {
        login!();

        let mut member = get_member();
        let guild = MOCK_GUILD_DATA.clone();
        let red_colour = guild.roles.get(&RED_COLOUR_ID).unwrap();
        let green_colour = guild.roles.get(&GREEN_COLOUR_ID).unwrap();

        member
            .add_role(green_colour)
            .expect("Couldn't assign a role to the user!");

        let guildrw = RwLock::new(guild.clone());

        let result = assign_colour_to_user(&member.user.read(), guildrw.write(), &red_colour, conn);

        let mut guild = guildrw.try_write().unwrap();

        let member = guild
            .members
            .get_mut(&UserId(CONFIG.discord.id.parse().unwrap()))
            .unwrap();

        let has_red = member.roles.contains(&red_colour.id);
        let has_green = member.roles.contains(&green_colour.id);

        let _ = member.remove_roles(&[red_colour.id, green_colour.id]);

        result.expect("assign_colour_to_user should have been Ok, instead got:");

        assert!(
            has_red,
            "User does not have red colour when it should have had the role!"
        );
        assert!(
            !has_green,
            "User has the green colour when that should have been removed during the assignment!"
        );
    })
}

#[test]
fn can_generate_colour_image() {
    do_test_transaction!(|conn| {
        let colours = find_all(&DB_GUILD, conn)
            .expect("failure while trying to get all the colours for a guild.");

        let result = generate_colour_image(&colours, &MOCK_GUILD_DATA)
            .expect("Error while generating the colour list.");

        let path_exists = fs::File::open(result.clone()).is_ok();

        // cleanup
        let _ = fs::remove_file(result);

        assert!(
            path_exists,
            "File should be in the OS, instead got an error!"
        );
    })
}

// keeps throwing an invalidpermissions error, all
// other gateway related things except this one
// fails.
#[test]
fn can_update_a_role_and_the_record() {
    use serenity::CACHE;

    do_test_transaction!(|conn| {
        login!();

        http::set_token(&format!("Bot {}", &CONFIG.discord.token));
        // IMPORTANT!
        // this updates the serenity cache with the
        // user that is currently logged in so that
        // it doesn't autofail the requests sent below.
        let mut cache = CACHE.write();
        cache.user = http::get_current_user().expect("failure at get current user");

        cache.guilds.insert(
            MOCK_GUILD_DATA.id,
            Arc::new(RwLock::new(MOCK_GUILD_DATA.clone())),
        );

        let cached_guild = cache
            .guilds
            .get(&MOCK_GUILD_DATA.id)
            .expect("failure at getting guild from cache")
            .clone();
        drop(cache);

        let mut cached_guild = cached_guild.write().clone();

        let role = cached_guild
            .create_role(|r| {
                r.name("test role")
                    .permissions(Permissions::empty())
                    .mentionable(false)
            }).expect("Error while creating the role for the test!");

        let colour_record =
            convert_role_to_record_struct("test".to_string(), &role, &MOCK_GUILD_DATA.id)
                .ok_or_else(|| {
                    role.delete()
                        .expect("failure at deleting role after failure");
                }).expect("Error while converting role to a record");

        let colour =
            save_record_to_db(colour_record, conn).expect("Error saving a record to the database.");

        cached_guild.roles.insert(role.id, role.clone());
        {
            let mut cache = CACHE.write();
            cache
                .guilds
                .insert(cached_guild.id, Arc::new(RwLock::new(cached_guild.clone())));
        }

        let new_colour = update_colour_and_role(
            UpdateActionParams {
                guild: &cached_guild,
                colour: colour,
                new_colour: Some("#00FF00".parse::<ParsedColour>().unwrap()),
                new_name: Some("updated test"),
                change_role_name: true,
            },
            conn,
        );

        let roles = http::get_guild_roles(MOCK_GUILD_DATA.id.0);

        http::delete_role(cached_guild.id.0, role.id.0)
            .expect("Failure deleting the role from the guild");

        let roles = roles.expect("couldn't get guild roles");

        let role = roles
            .iter()
            .find(|e| &e.id == &role.id)
            .expect("role did not get updated on discord");

        let new_colour = new_colour.expect("Error while updating role and record!");

        assert_eq!(role.name, "updated test", "Role name was not updated!");
        assert_eq!(
            new_colour.name, "updated test",
            "Record name was not updated!"
        );
    })
}
