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
