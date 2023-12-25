use std::sync::Arc;

use plugins::PluginResult;
use userinput::UserInput;

pub mod plugindispatcher;
pub mod plugins;
mod pluginworker;
pub mod userinput;
pub mod util;

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Arc<dyn PluginResult>>),
    UserInput(Arc<UserInput>),
    RemoveWindow,
    ChangeSelect(u32),
    SelectSomething,
}
