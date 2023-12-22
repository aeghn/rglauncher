use std::sync::Arc;

use plugins::PluginResult;
use userinput::UserInput;

pub mod plugins;
pub mod util;
pub mod userinput;
mod plugindispatcher;
mod pluginworker;


pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Box<dyn PluginResult>>),
    UserInput(Arc<UserInput>),
    RemoveWindow(i32),
}