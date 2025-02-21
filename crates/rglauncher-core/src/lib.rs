use plugins::PRWrapper;
use userinput::{Signal, UserInput};

pub mod config;
pub mod dispatcher;
pub mod plugins;
pub mod userinput;
pub mod util;

pub enum ResultMsg {
    Result(Signal, Vec<PRWrapper>),
    UserInput(UserInput),
    ChangeSelect(u32),
    SelectSomething,
}
