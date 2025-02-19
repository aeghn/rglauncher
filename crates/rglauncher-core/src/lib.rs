use std::sync::Arc;

use plugins::PRWrapper;
use userinput::UserInput;

pub mod config;
pub mod dispatcher;
pub mod plugins;
pub mod userinput;
pub mod util;

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<PRWrapper>),
    UserInput(UserInput),
    RemoveWindow,
    ChangeSelect(u32),
    SelectSomething,
}
