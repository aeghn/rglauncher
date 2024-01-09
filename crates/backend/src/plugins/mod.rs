pub mod app;
pub mod calculator;
pub mod clipboard;
pub mod dict;
pub mod windows;
pub mod history;

use crate::userinput::UserInput;
use crate::ResultMsg;
use std::any::Any;
use std::sync::Arc;

pub enum PluginMsg<T> {
    UserInput(Arc<UserInput>, flume::Sender<ResultMsg>),
    RefreshContent,
    TypeMsg(T),
}

#[typetag::serde(tag = "type")]
pub trait PluginResult: Send + Sync {
    fn score(&self) -> i32;

    fn sidebar_icon_name(&self) -> String;

    fn sidebar_label(&self) -> Option<String>;

    fn sidebar_content(&self) -> Option<String>;

    fn on_enter(&self);

    fn get_type_id(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn get_id(&self) -> &str;
}

// TODO: async trait
pub trait Plugin<R, T>
where
    R: PluginResult,
    T: Send,
{
    fn handle_msg(&mut self, msg: T);

    fn refresh_content(&mut self);

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<R>>;

    fn get_type_id(&self) -> &'static str;
}
