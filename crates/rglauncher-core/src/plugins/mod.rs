pub mod application;
#[cfg(feature = "calc")]
pub mod calculator;
#[cfg(feature = "clip")]
pub mod clipboard;
#[cfg(feature = "mdict")]
pub mod dictionary;
pub mod history;
#[cfg(feature = "hyprwin")]
pub mod windows;

use crate::userinput::UserInput;
use std::any::Any;

use self::history::HistoryItem;

pub trait PluginResult: Send + Sync {
    fn score(&self) -> i32;

    fn icon_name(&self) -> &str;

    fn name(&self) -> &str;

    fn extra(&self) -> Option<&str>;

    fn on_enter(&self);

    fn get_type_id(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn get_id(&self) -> &str;
}

// TODO: async trait
pub trait Plugin<R, T>: Send
where
    R: PluginResult,
    T: Send,
{
    fn handle_msg(&mut self, msg: T);

    fn refresh_content(&mut self);

    fn handle_input(
        &self,
        user_input: &UserInput,
        history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<R>>;

    fn get_type_id(&self) -> &'static str;
}
