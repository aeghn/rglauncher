pub mod app;
pub mod windows;
pub mod clipboard;


use crate::shared::UserInput;

pub trait PluginResult {
    /*
     * 获取匹配得分
     */
    fn get_score(&self) -> i32;

    fn sidebar_icon(&self) -> Option<gio::Icon>;

    fn sidebar_label(&self) -> Option<String>;

    fn sidebar_content(&self) -> Option<gtk::Widget>;

    fn preview(&self) -> gtk::Grid;

    fn on_enter(&self);
}

pub trait Plugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<Box<dyn PluginResult + Send>>;
}
