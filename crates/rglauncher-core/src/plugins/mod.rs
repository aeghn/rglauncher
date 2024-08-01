pub mod application;
#[cfg(feature = "calc")]
pub mod calculator;
#[cfg(feature = "clip")]
pub mod clipboard;
#[cfg(feature = "hyprwin")]
pub mod hyprwindows;
#[cfg(feature = "mdict")]
pub mod mdict;

use application::freedesktop::{FDAppItem, FDAppPlugin};
use calculator::{CalcItem, CalculatorPlugin};
use clipboard::{ClipItem, ClipboardPlugin};
use enum_dispatch::enum_dispatch;
use hyprwindows::{HyprWindowItem, HyprWindowsPlugin};
use mdict::{MDictItem, MDictPlugin};

use crate::userinput::UserInput;

#[derive(Clone)]
#[enum_dispatch]
pub enum PluginItem {
    MDict(MDictItem),
    HyprWin(HyprWindowItem),
    Clip(ClipItem),
    Calc(CalcItem),
    App(FDAppItem),
}

#[enum_dispatch(PluginItem)]
pub trait PluginItemTrait: Send + Sync {
    fn get_score(&self) -> i32;
    fn on_activate(&self);
    fn get_type(&self) -> &'static str;
    fn get_id(&self) -> &str;
}

pub trait PluginTrait {
    type Msg;
    type Item: PluginItemTrait;

    fn get_type(&self) -> &'static str;

    async fn refresh(&mut self) {}
    async fn handle_msg(&mut self, _msg: Self::Msg) {}
    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<Self::Item>>;
}

pub enum Plugin {
    MDict(MDictPlugin),
    HyprWin(HyprWindowsPlugin),
    Clip(ClipboardPlugin),
    Calc(CalculatorPlugin),
    FDApp(FDAppPlugin),
}

impl PluginTrait for Plugin {
    type Msg = ();

    type Item = PluginItem;

    fn get_type(&self) -> &'static str {
        match self {
            Plugin::MDict(e) => e.get_type(),
            Plugin::HyprWin(e) => e.get_type(),
            Plugin::Clip(e) => e.get_type(),
            Plugin::Calc(e) => e.get_type(),
            Plugin::FDApp(e) => e.get_type(),
        }
    }

    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<Self::Item>> {
        match self {
            Plugin::MDict(e) => e
                .handle_input(user_input)
                .await
                .map(|v| v.into_iter().map(|r| r.into()).collect()),
            Plugin::HyprWin(e) => e
                .handle_input(user_input)
                .await
                .map(|v| v.into_iter().map(|r| r.into()).collect()),
            Plugin::Clip(e) => e
                .handle_input(user_input)
                .await
                .map(|v| v.into_iter().map(|r| r.into()).collect()),
            Plugin::Calc(e) => e
                .handle_input(user_input)
                .await
                .map(|v| v.into_iter().map(|r| r.into()).collect()),
            Plugin::FDApp(e) => e
                .handle_input(user_input)
                .await
                .map(|v| v.into_iter().map(|r| r.into()).collect()),
        }
    }
}

impl Into<Plugin> for MDictPlugin {
    fn into(self) -> Plugin {
        Plugin::MDict(self)
    }
}

impl Into<Plugin> for ClipboardPlugin {
    fn into(self) -> Plugin {
        Plugin::Clip(self)
    }
}

impl Into<Plugin> for HyprWindowsPlugin {
    fn into(self) -> Plugin {
        Plugin::HyprWin(self)
    }
}

impl Into<Plugin> for CalculatorPlugin {
    fn into(self) -> Plugin {
        Plugin::Calc(self)
    }
}

impl Into<Plugin> for application::freedesktop::FDAppPlugin {
    fn into(self) -> Plugin {
        Plugin::FDApp(self)
    }
}
