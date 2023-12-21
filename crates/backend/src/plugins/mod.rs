pub mod app;
pub mod calculator;
pub mod clipboard;
pub mod dict;
pub mod windows;


use std::sync::Arc;
use crate::userinput::UserInput;

pub enum PluginMsg<T> {
    UserInput(Arc<UserInput>),
    RefreshContent,
    TypeMsg(T),
}

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Box<dyn PluginResult>>),
    UserInput(Arc<UserInput>),
    RemoveWindow(i32),
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
    fn handle_msg(&mut self, msg: T);

    fn refresh_content(&mut self);

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<R>>;
}