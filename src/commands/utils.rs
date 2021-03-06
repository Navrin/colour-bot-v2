use serenity::client::Context;
use serenity::framework::standard::CreateCommand;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::prelude::{Message, User};
use serenity::utils::Colour;
use serenity::CACHE;

pub fn info(cmd: CreateCommand) -> CreateCommand {
    cmd.desc("Displays some useful infomation about the bot in embed form.")
        .exec(info_exec)
}

pub fn info_exec(_ctx: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
    let cache = CACHE.read();

    let channel = msg
        .channel()
        .ok_or_else(|| CommandError("Channel does not exist!".to_string()))?;

    let guilds = cache.all_guilds();
    let guild_count = guilds.len();

    let user_count = guilds
        .iter()
        .filter_map(|guild| guild.members::<User>(None, None).map(|r| r.len()).ok())
        .sum::<usize>();

    // let colour_count = <<TODO>>!
    // let requests_performed = <<TODO>>!

    // is of (Field name, field content, inline)
    let fields = vec![
        ("Guild count", guild_count.to_string(), true),
        ("Total users", user_count.to_string(), true),
    ];

    channel.id().send_message(|msg| {
        msg.embed(|embed| {
            embed
                .title("Colour Bot V2")
                .colour(Colour::RED)
                .fields(fields)
        })
    })?;

    Ok(())
}
