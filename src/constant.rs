use std::sync::Mutex;

pub const PLUGIN_RESULT_LOCK: Mutex<u8> = Mutex::new(0);
pub const STORE_DB : &str= &"/home/chin/.cache/rglauncher/store.db";