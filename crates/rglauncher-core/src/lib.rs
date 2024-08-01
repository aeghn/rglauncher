use std::sync::Arc;

use plugins::PluginItemTrait;
use userinput::UserInput;

pub mod config;
pub mod misc;
pub mod plugins;
pub mod userinput;
pub mod util;
pub mod history;
pub mod db;

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Arc<dyn PluginItemTrait>>),
    UserInput(Arc<UserInput>),
    RemoveWindow,
    ChangeSelect(u32),
    SelectSomething,
}
