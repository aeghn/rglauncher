use std::process::Command;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tracing::info;

use crate::plugins::{PluginItemTrait, PluginTrait};
use crate::userinput::UserInput;

use crate::util::scoreutils;

pub const TYPE: &str = "hypr_windows";

#[derive(Clone)]
pub enum HyprWindowMsg {}

#[derive(Clone)]
pub struct HyprWindowItem {
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

impl PluginItemTrait for HyprWindowItem {
    fn get_score(&self) -> i32 {
        scoreutils::high(self.score as i64)
    }

    fn on_activate(&self) {
        // dispatch focuswindow address:
        let _msg = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg("address:".to_owned() + self.address.as_str())
            .output()
            .expect("unable to switch to the window");
    }

    fn get_type(&self) -> &'static str {
        &TYPE
    }

    fn get_id(&self) -> &str {
        self.address.as_str()
    }
}

pub struct HyprWindowsPlugin {
    windows: Vec<HyprWindowItem>,
}

impl HyprWindowsPlugin {
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating Windows Plugin(Hyprland)");

        Ok(HyprWindowsPlugin {
            windows: get_windows()?,
        })
    }
}

fn get_windows() -> anyhow::Result<Vec<HyprWindowItem>> {
    let output = Command::new("hyprctl").arg("clients").arg("-j").output()?;

    let out = String::from_utf8(output.stdout)?;

    let json = serde_json::from_str::<serde_json::Value>(out.as_str())
        .unwrap_or_else(|_| serde_json::Value::Null);

    if let Some(array) = json.as_array() {
        let vec: Vec<HyprWindowItem> = array
            .iter()
            .filter_map(|e| {
                let class = e.get("class")?.as_str()?;
                let monitor = e.get("monitor")?.as_i64()?;
                if monitor == -1 {
                    return None;
                }

                Some(HyprWindowItem {
                    class: class.to_string(),
                    title: e.get("title")?.as_str()?.to_string(),
                    address: e.get("address")?.as_str()?.to_string(),
                    mapped: e.get("mapped")?.as_bool()?,
                    hidden: e.get("hidden")?.as_bool()?,
                    pid: e.get("pid")?.as_i64()?,
                    xwayland: e.get("xwayland")?.as_bool()?,
                    monitor: monitor,
                    workspace: e.get("workspace")?.get("name")?.as_str()?.to_string(),
                    score: 0,
                })
            })
            .collect();
        Ok(vec)
    } else {
        anyhow::bail!("hyprctl out is not a valid json")
    }
}

impl PluginTrait for HyprWindowsPlugin {
    type Msg = HyprWindowMsg;

    type Item = HyprWindowItem;

    async fn refresh(&mut self) {
        info!("update windows");
        if let Ok(windows) = get_windows() {
            self.windows = windows;
        }
    }

    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<HyprWindowItem>> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<HyprWindowItem> = vec![];

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

    fn get_type(&self) -> &'static str {
        &TYPE
    }
}
