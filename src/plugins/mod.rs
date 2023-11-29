pub mod app;
pub mod clipboard;
pub mod dict;
pub mod windows;
pub mod calculator;

use crate::user_input::UserInput;

pub trait PluginResult: Send {
    /*
     * 获取匹配得分
     */
    fn get_score(&self) -> i32;

    fn sidebar_icon_name(&self) -> String;

    fn sidebar_label(&self) -> Option<String>;

    fn sidebar_content(&self) -> Option<String>;

    fn preview(&self) -> gtk::Widget;

    fn on_enter(&self);
}

pub trait Plugin<R: PluginResult> {
    fn handle_input(&self, user_input: &UserInput) -> Vec<R>;
}
