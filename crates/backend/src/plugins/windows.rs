use std::process::Command;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tracing::info;

use crate::plugins::{Plugin, PluginResult};
use crate::plugins::history::HistoryItem;
use crate::userinput::UserInput;

use crate::util::score_utils;

pub const TYPE_ID: &str = "hypr_windows";

#[derive(Clone)]
pub enum HyprWindowMsg {}

#[derive(Clone)]
pub struct HyprWindowResult {
    pub class: String,
    pub title: String,
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub pid: i64,
    pub xwayland: bool,
    pub monitor: i64,
    pub workspace: String,
    pub score: i32,
}

impl PluginResult for HyprWindowResult {
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
        // dispatch focuswindow address:
        let _msg = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg("address:".to_owned() + self.address.as_str())
            .output()
            .expect("unable to switch to the window");
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

pub struct HyprWindowsPlugin {
    windows: Vec<HyprWindowResult>,
}

impl HyprWindowsPlugin {
    pub fn new() -> Self {
        info!("Creating Windows Plugin");

        HyprWindowsPlugin {
            windows: get_windows(),
        }
    }
}

fn get_windows() -> Vec<HyprWindowResult> {
    let output = Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .unwrap();
    let mut vec: Vec<HyprWindowResult> = vec![];

    let out = String::from_utf8(output.stdout).unwrap();

    let json = serde_json::from_str::<serde_json::Value>(out.as_str())
        .unwrap_or_else(|_| serde_json::Value::Null);

    if let Some(array) = json.as_array() {
        for e in array {
            let class = e.get("class").unwrap().as_str().unwrap();
            let monitor = e.get("monitor").unwrap().as_i64().unwrap();
            if monitor == -1 {
                continue;
            }

            vec.push(HyprWindowResult {
                class: class.to_string(),
                title: e.get("title").unwrap().as_str().unwrap().to_string(),
                address: e.get("address").unwrap().as_str().unwrap().to_string(),
                mapped: e.get("mapped").unwrap().as_bool().unwrap(),
                hidden: e.get("hidden").unwrap().as_bool().unwrap(),
                pid: e.get("pid").unwrap().as_i64().unwrap(),
                xwayland: e.get("xwayland").unwrap().as_bool().unwrap(),
                monitor: monitor,
                workspace: e
                    .get("workspace")
                    .unwrap()
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                score: 0,
            })
        }
    }

    vec
}

impl Plugin<HyprWindowResult, HyprWindowMsg> for HyprWindowsPlugin {
    fn refresh_content(&mut self) {
        info!("update windows");
        self.windows = get_windows();
    }

    fn handle_input(&self, user_input: &UserInput, history: Option<Vec<&HistoryItem>>) -> anyhow::Result<Vec<HyprWindowResult>> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<HyprWindowResult> = vec![];

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

    fn handle_msg(&mut self, msg: HyprWindowMsg) {
        todo!()
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
