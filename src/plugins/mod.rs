pub mod app;
pub mod calculator;
pub mod clipboard;
pub mod dict;
pub mod factory;
pub mod windows;

use crate::userinput::UserInput;

pub trait PluginResult: Send {
    fn score(&self) -> i32;

    fn sidebar_icon_name(&self) -> String;

    fn sidebar_label(&self) -> Option<String>;

    fn sidebar_content(&self) -> Option<String>;

    fn on_enter(&self);
}

pub trait Plugin<R>
where
    R: PluginResult,
{
    fn refresh_content(&mut self);

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<R>>;
}

pub trait PluginPreview<R>: 'static
where
    R: PluginResult,
{
    fn new() -> Self
    where
        Self: Sized;

    fn get_preview(&self, plugin_result: R) -> gtk::Widget;
}
