/// Macro to delete a message on a seperate thread after some time
/// supports multiple messages with (msg1, msg2, msg3; 1)
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
