use serenity::framework::standard::CreateCommand;
use serenity::model::prelude::{Message, User};
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandError};

pub fn get_colour(cmd: CreateCommand) -> CreateCommand {
    cmd.batch_known_as(&["getc", "getcolour", "getcolor", "colour", "color"])
        .exec(get_colour_exec)
}

pub fn get_colour_exec(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), CommandError> {
    Ok(())
}
