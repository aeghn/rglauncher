pub mod app;
#[cfg(feature = "calc")]
pub mod calc;
#[cfg(feature = "clip")]
pub mod clip;
pub mod history;
#[cfg(feature = "mdict")]
pub mod mdict;
#[cfg(feature = "wmwin")]
pub mod win;

use std::ops::Deref;

use app::AppPlugin;
use calc::CalcPlugin;
use chin_tools::AResult;
#[cfg(feature = "clip")]
use clip::ClipPlugin;
#[cfg(feature = "fmdict")]
use mdict::DictPlugin;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use win::WinPlugin;

use crate::plugins::app::{AppReq, AppResult};
#[cfg(feature = "calc")]
use crate::plugins::calc::{CalcReq, CalcResult};
#[cfg(feature = "clip")]
use crate::plugins::clip::{ClipReq, ClipResult};
#[cfg(feature = "mdict")]
use crate::plugins::mdict::{DictMsg, DictResult};
#[cfg(feature = "wmwin")]
use crate::plugins::win::{WinResult, WindowMsg};

use crate::userinput::UserInput;

pub trait Plugin: Send + Sync {
    type R: PluginResult;
    type T: Send;

    fn handle_msg(&self, _msg: Self::T) {}

    fn refresh_content(&self) {}

    fn handle_input(&self, user_input: &UserInput) -> AResult<Vec<(Self::R, i32)>>;

    fn get_type_id(&self) -> &'static str;
}

pub enum PluginEnum {
    App(AppPlugin),
    #[cfg(feature = "calc")]
    Calc(CalcPlugin),
    #[cfg(feature = "clip")]
    Clip(ClipPlugin),
    #[cfg(feature = "mdict")]
    Dict(DictPlugin),
    #[cfg(feature = "wmwin")]
    Win(WinPlugin),
}

macro_rules! pimpl {
    ($self:expr, $($tt:tt)*) => {{
        match $self {
            #[cfg(feature = "mdict")]
            PluginEnum::Dict(r) => {r.$($tt)*}
            #[cfg(feature = "calc")]
            PluginEnum::Calc(r) => {r.$($tt)*}
            PluginEnum::App(r) => {r.$($tt)*}
            #[cfg(feature = "wmwin")]
            PluginEnum::Win(r) => {r.$($tt)*}
            #[cfg(feature = "clip")]
            PluginEnum::Clip(r) => {r.$($tt)*}
        }
    }};
}

impl Plugin for PluginEnum {
    type R = PluginResultEnum;

    type T = PluginReqEnum;

    fn handle_input(&self, user_input: &UserInput) -> AResult<Vec<(Self::R, i32)>> {
        Ok(pimpl!(
            self,
            handle_input(user_input)?
                .into_iter()
                .map(|(pr, s)| (pr.to_enum(), s))
                .collect()
        ))
    }

    fn get_type_id(&self) -> &'static str {
        pimpl!(self, get_type_id())
    }
}

pub enum PluginReqEnum {
    App(AppReq),
    #[cfg(feature = "calc")]
    Calc(CalcReq),
    #[cfg(feature = "clip")]
    Clip(ClipReq),
    #[cfg(feature = "mdict")]
    Dict(DictMsg),
    #[cfg(feature = "wmwin")]
    Win(WindowMsg),
}

pub trait PluginResult: Send + Sync + Clone + DeserializeOwned + Serialize {
    fn icon_name(&self) -> &str;

    fn name(&self) -> &str;

    fn extra(&self) -> Option<&str>;

    fn on_enter(&self);

    fn get_type_id(&self) -> &'static str;

    fn get_id(&self) -> &str;

    fn to_enum(self) -> PluginResultEnum;
}

#[derive(Clone, Deserialize, Serialize)]
pub enum PluginResultEnum {
    App(AppResult),
    #[cfg(feature = "mdict")]
    MDict(DictResult),
    #[cfg(feature = "calc")]
    Calc(CalcResult),
    #[cfg(feature = "wmwin")]
    Win(WinResult),
    #[cfg(feature = "clip")]
    Clip(ClipResult),
}

#[derive(Clone)]
pub struct PRWrapper {
    pub body: PluginResultEnum,
    pub score: i32,
}

impl Deref for PRWrapper {
    type Target = PluginResultEnum;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl<T> Into<PRWrapper> for (T, i32)
where
    T: PluginResult,
{
    fn into(self) -> PRWrapper {
        PRWrapper {
            body: self.0.to_enum(),
            score: self.1,
        }
    }
}

macro_rules! primpl {
    ($self:expr, $method:ident) => {
        match $self {
            PluginResultEnum::App(r) => r.$method(),
            #[cfg(feature = "mdict")]
            PluginResultEnum::MDict(r) => r.$method(),
            #[cfg(feature = "calc")]
            PluginResultEnum::Calc(r) => r.$method(),
            #[cfg(feature = "wmwin")]
            PluginResultEnum::Win(r) => r.$method(),
            #[cfg(feature = "clip")]
            PluginResultEnum::Clip(r) => r.$method(),
        }
    };
}

impl PluginResult for PluginResultEnum {
    fn icon_name(&self) -> &str {
        primpl!(self, icon_name)
    }

    fn name(&self) -> &str {
        primpl!(self, name)
    }

    fn extra(&self) -> Option<&str> {
        primpl!(self, extra)
    }

    fn on_enter(&self) {
        primpl!(self, on_enter)
    }

    fn get_type_id(&self) -> &'static str {
        primpl!(self, get_type_id)
    }

    fn get_id(&self) -> &str {
        primpl!(self, get_id)
    }

    fn to_enum(self) -> PluginResultEnum {
        primpl!(self, to_enum)
    }
}
