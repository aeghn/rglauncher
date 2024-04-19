use std::sync::Arc;

use plugins::PluginResult;
use userinput::UserInput;

pub mod dispatcher;
pub mod plugins;
pub mod userinput;
pub mod util;
pub mod config;

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Arc<dyn PluginResult>>),
    UserInput(Arc<UserInput>),
    RemoveWindow,
    ChangeSelect(u32),
    SelectSomething,
}
