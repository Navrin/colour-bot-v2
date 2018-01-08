/// This section contains the "actions" for the bot.
/// These are split up into functions that perform the **LEAST** amount of actions needed, like changing a DB record to a discord role.
/// These functions should be easily composible so they can be chained togeather in the command section.
/// No monolith functions! All these functions should be their own isolated contexts.