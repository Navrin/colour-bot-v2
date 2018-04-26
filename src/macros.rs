/// Macro to delete a message on a seperate thread after some time
/// supports multiple messages with `delay_delete!(msg1, msg2, msg3; 1)`
/// time is set in second after a `;`
#[macro_export]
macro_rules! delay_delete {
    ($($msg:expr),+; $time:expr) => {
        use ::std::thread;
        use ::std::time::Duration;

        thread::spawn(move || {
            thread::sleep(Duration::from_secs($time));
            $(let _ = $msg.delete();)+
        });
    };
}

#[macro_export]
macro_rules! reply_error_closure {
    ($msg:expr) => {{
        let msg = $msg.clone();

        || {
            use serenity::framework::standard::CommandError;
            delay_delete!(msg; 5);
            CommandError("No reply recieved within 15 seconds, giving up!".to_string())
        }
    }};
}

#[macro_export]
macro_rules! discord_api_url {
    () => {
        "https://discordapp.com/api/v6"
    };
}
