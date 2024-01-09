use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use fork::Fork;
use freedesktop_desktop_entry::DesktopEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use lazy_static::lazy_static;
use regex::Regex;
use std::option::Option::None;
use std::process::{exit, Command, Stdio};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::util::score_utils;

pub enum AppMsg {}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct AppResult {
    pub icon_name: String,
    pub app_name: String,
    pub app_desc: String,
    pub exec: String,
    score: i32,
    pub id: String,
    pub desktop_path: String,
    pub terminal: bool,
}

lazy_static! {
    static ref PLACE_HOLDER_REPLACER: Regex = Regex::new(r"%\w").unwrap();
}

pub const TYPE_ID: &str = "app_result";

fn run_command(command: Vec<&str>) {
    match fork::fork() {
        Ok(Fork::Child) => match fork::fork() {
            Ok(Fork::Child) => {
                fork::setsid().expect("Failed to setsid");
                match Command::new(&command[0])
                    .args(&command[1..])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(child) => exit(0),
                    Err(err) => {
                        error!("Error running command: {}", err);
                        exit(1)
                    }
                }
            }
            Err(e) => {
                error!("unable to run command: {:?} {}", command, e)
            }
            _ => {}
        },
        Err(e) => {
            error!("unable running command: {}", e)
        }
        _ => {}
    }
}

#[typetag::serde]
impl PluginResult for AppResult {
    fn score(&self) -> i32 {
        score_utils::high(self.score as i64)
    }

    fn sidebar_icon_name(&self) -> String {
        self.icon_name.clone()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.app_name.clone())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(self.app_desc.clone())
    }

    fn on_enter(&self) {
        let mut true_command = PLACE_HOLDER_REPLACER
            .replace_all(self.exec.as_str(), "")
            .trim()
            .to_string();

        if self.terminal {
            Command::new("foot")
                .arg("-e")
                .arg(true_command)
                .spawn()
                .expect("unable to spawn terminal app");
        } else {
            let cmd_and_args: Vec<&str> = true_command.split(" ").collect();
            run_command(cmd_and_args);
        }
    }

    fn get_type_id(&self) -> &'static str {
        TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn get_id(&self) -> &str {
        &self.desktop_path
    }
}

pub struct ApplicationPlugin {
    applications: Vec<AppResult>,
    matcher: SkimMatcherV2,
}

impl ApplicationPlugin {
    pub fn new() -> Self {
        info!("Creating App Plugin");
        let matcher = SkimMatcherV2::default();

        let applications =
            freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
                .into_iter()
                .filter_map(|path| {
                    if let Ok(bytes) = std::fs::read_to_string(&path) {
                        if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                            if entry.no_display() {
                                return None;
                            }

                            return Some(AppResult {
                                id: entry.id().to_string(),
                                icon_name: entry.icon().unwrap_or_default().to_string(),
                                app_name: entry.name(None).unwrap_or_default().to_string(),
                                app_desc: entry.comment(None).unwrap_or_default().to_string(),
                                exec: entry.exec().unwrap_or_default().to_string(),
                                desktop_path: path.to_str().unwrap_or_default().to_string(),
                                terminal: entry.terminal(),
                                score: 0,
                            });
                        }
                    }
                    None
                })
                .collect();

        ApplicationPlugin {
            applications,
            matcher,
        }
    }
}

impl Plugin<AppResult, AppMsg> for ApplicationPlugin {
    fn handle_msg(&mut self, msg: AppMsg) {}

    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<AppResult>> {
        let result = self
            .applications
            .iter()
            .filter_map(|app| {
                if user_input.input.is_empty()   {
                    return Some(app.clone())
                }

                let score = self.matcher.fuzzy_match(&app.app_name, &user_input.input);

                if score.unwrap_or(0) > 0 {
                    Some(app.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        plugins::{app::ApplicationPlugin, Plugin},
        userinput::UserInput,
    };

    #[test]
    fn test_app() {
        let app_plugin = ApplicationPlugin::new();
        println!("apps: {:?}", app_plugin.handle_input(&UserInput::new("a")));
    }
}
