use std::process::{Command, ExitStatus};

use anyhow::Context;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tracing::{error, info};

use crate::plugins::history::HistoryItem;
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;

use crate::util::score_utils;

pub const TYPE_ID: &str = "wmwindows";

#[derive(Clone, Debug)]
pub enum WMEnum {
    Niri,
    Hypr,
}
pub trait WMBehavier {
    fn focus_window(&self, id: &str) -> anyhow::Result<()>;
    fn list_windows(&self) -> anyhow::Result<Vec<WMWindowResult>>;
}

impl WMBehavier for WMEnum {
    fn focus_window(&self, id: &str) -> anyhow::Result<()> {
        match self {
            WMEnum::Niri => {
                let output = Command::new("niri")
                    .arg("msg")
                    .arg("action")
                    .arg("focus-window")
                    .arg("--id")
                    .arg(id)
                    .output()?;
                if (!output.status.success()) {
                    error!("unable to success: {:?}", output);
                }
            }
            WMEnum::Hypr => {
                let output = Command::new("hyprctl")
                    .arg("dispatch")
                    .arg("focuswindow")
                    .arg("address:".to_owned() + id)
                    .output()?;
                if (!output.status.success()) {
                    error!("unable to success: {:?}", output);
                }
            }
        }
        Ok(())
    }

    fn list_windows(&self) -> anyhow::Result<Vec<WMWindowResult>> {
        match self {
            WMEnum::Niri => {
                let output = Command::new("niri")
                    .arg("msg")
                    .arg("-j")
                    .arg("windows")
                    .output()?;

                let out = String::from_utf8(output.stdout)?;

                let json = serde_json::from_str::<serde_json::Value>(out.as_str())
                    .unwrap_or_else(|_| serde_json::Value::Null);

                let vec: Vec<WMWindowResult> = json
                    .as_array()
                    .context("hyprctl output is not a valid json")?
                    .iter()
                    .filter_map(|e| {
                        info!("-> {:?}", e.get("id")?.as_i64()?);
                        let a = Some(WMWindowResult {
                            class: e.get("app_id")?.as_str()?.to_string(),
                            title: e.get("title")?.as_str()?.to_string(),
                            address: e.get("id")?.as_i64()?.to_string(),
                            pid: e.get("pid")?.as_i64()?,
                            workspace: e.get("workspace_id")?.as_i64()?.to_string(),
                            score: score_utils::high(1),
                            wm_type: WMEnum::Niri,
                        });
                        a
                    })
                    .collect();

                tracing::info!("windows: {:?}", vec);

                Ok(vec)
            }
            WMEnum::Hypr => {
                let output = Command::new("hyprctl").arg("clients").arg("-j").output()?;
                let out = String::from_utf8(output.stdout)?;
                let json = serde_json::from_str::<serde_json::Value>(out.as_str())?;

                let array = json
                    .as_array()
                    .context("hyprctl output is not a valid json")?;
                let vec: Vec<WMWindowResult> = array
                    .iter()
                    .filter_map(|e| {
                        let class = e.get("class")?.as_str()?;
                        let monitor = e.get("monitor")?.as_i64()?;
                        if monitor == -1 {
                            return None;
                        }

                        Some(WMWindowResult {
                            class: class.to_string(),
                            title: e.get("title")?.as_str()?.to_string(),
                            address: e.get("address")?.as_str()?.to_string(),
                            pid: e.get("pid")?.as_i64()?,
                            workspace: e.get("workspace")?.get("name")?.as_str()?.to_string(),
                            score: score_utils::high(1),
                            wm_type: WMEnum::Hypr,
                        })
                    })
                    .collect();
                Ok(vec)
            }
        }
    }
}

#[derive(Clone)]
pub enum WMWindowMsg {}

#[derive(Clone, Debug)]
pub struct WMWindowResult {
    pub class: String,
    pub title: String,
    pub address: String,
    pub pid: i64,
    pub workspace: String,
    pub score: i32,
    pub wm_type: WMEnum,
}

impl PluginResult for WMWindowResult {
    fn score(&self) -> i32 {
        score_utils::high(self.score as i64)
    }

    fn icon_name(&self) -> &str {
        self.class.as_str()
    }

    fn name(&self) -> &str {
        self.title.as_str()
    }

    fn extra(&self) -> Option<&str> {
        Some(self.workspace.as_str())
    }

    fn on_enter(&self) {
        if let Err(e) = self.wm_type.focus_window(&self.address) {
            error!("unable to focus on {}", e);
        }
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn get_id(&self) -> &str {
        self.address.as_str()
    }
}

pub struct WMWindowsPlugin {
    windows: Vec<WMWindowResult>,
    wm_type: WMEnum,
}

impl WMWindowsPlugin {
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Windows Plugin");
        let wm_type = WMEnum::Niri;

        Ok(WMWindowsPlugin {
            windows: wm_type.list_windows()?,
            wm_type,
        })
    }
}

impl Plugin<WMWindowResult, WMWindowMsg> for WMWindowsPlugin {
    fn refresh_content(&mut self) {
        info!("update windows");
        if let Ok(windows) = self.wm_type.list_windows() {
            self.windows = windows;
        }
    }

    fn handle_input(
        &self,
        user_input: &UserInput,
        _history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<WMWindowResult>> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<WMWindowResult> = vec![];

        for window in &self.windows {
            let mut score: i32 = 0;
            let mut mstr = window.class.to_string();
            mstr += window.title.as_str();
            mstr += window.workspace.as_str();
            if user_input.input.is_empty() {
                score = 100;
            } else if let Some(_s) = matcher.fuzzy_match(mstr.as_str(), user_input.input.as_str()) {
                score = _s as i32;
            }

            let mut mw = window.clone();
            mw.score = score;
            if score > 0 {
                result.push(mw);
            }
        }

        Ok(result)
    }

    fn handle_msg(&mut self, _msg: WMWindowMsg) {
        todo!()
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
