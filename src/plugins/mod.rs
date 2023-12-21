pub mod app;
pub mod calculator;
pub mod clipboard;
pub mod dict;
pub mod windows;
mod plugindispatcher;
mod pluginworker;

use std::sync::Arc;
use crate::userinput::UserInput;

pub enum PluginMsg<T> {
    UserInput(Arc<UserInput>),
    RefreshContent,
    TypeMsg(T),
}

pub trait PluginResult: Send {
    fn score(&self) -> i32;

    fn sidebar_icon_name(&self) -> String;

    fn sidebar_label(&self) -> Option<String>;

    fn sidebar_content(&self) -> Option<String>;

    fn on_enter(&self);

    fn get_type_id(&self) -> &'static str;
}

// TODO: async trait
pub trait Plugin<R, T>
where
    R: PluginResult,
    T: Send
{
    fn refresh_content(&mut self);

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<R>>;

    fn handle_msg(msg: T);
}

pub trait PluginPreview {
    type PluginResult: PluginResult;

    fn new() -> Self
    where
        Self: Sized;

    fn get_preview(&self, plugin_result: &Self::PluginResult) -> gtk::Widget;
}
