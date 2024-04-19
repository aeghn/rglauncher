use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::util::string_utils::parse_cmd_string;
use freedesktop_desktop_entry::DesktopEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::option::Option::None;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tracing::{error, info};

use crate::util::score_utils;

use super::history::HistoryItem;

#[derive(Clone)]
pub enum AppMsg {}

#[derive(Debug, Clone)]
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

//https://github.com/alacritty/alacritty/blob/f7811548ae9cabb1122f43b42fec4d660318bc96/alacritty/src/daemon.rs#L28
fn run_command(command_and_args: Vec<&str>) -> io::Result<()> {
    let mut command = Command::new(&command_and_args[0]);
    command
        .args(&command_and_args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        command
            .pre_exec(|| {
                match libc::fork() {
                    -1 => return Err(io::Error::last_os_error()),
                    0 => (),
                    _ => libc::_exit(0),
                }

                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            })
            .spawn()?
            .wait()
            .map(|_| ())
    }
}

impl PluginResult for AppResult {
    fn score(&self) -> i32 {
        self.score
    }

    fn icon_name(&self) -> &str {
        self.icon_name.as_str()
    }

    fn name(&self) -> &str {
        self.app_name.as_str()
    }

    fn extra(&self) -> Option<&str> {
        Some(self.app_desc.as_str())
    }

    fn on_enter(&self) {
        if self.terminal {
            let true_command = PLACE_HOLDER_REPLACER
                .replace_all(self.exec.as_str(), "")
                .trim()
                .to_string();
            Command::new("foot")
                .arg("-e")
                .arg(true_command)
                .spawn()
                .expect("unable to spawn terminal app");
        } else {
            let true_command: Vec<String> = parse_cmd_string(self.exec.as_str())
                .into_iter()
                .filter(|e| !e.starts_with("%"))
                .collect();
            info!("exec command: {:?}", true_command);
            match run_command(true_command.iter().map(|e| e.as_str()).collect())
                .map_err(|e| error!("unable to start {}", e))
            {
                Ok(_) => {}
                Err(_) => {
                    error!("Unable to exec command: {:?}", true_command);
                }
            }
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
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating App Plugin");
        let matcher = SkimMatcherV2::default();

        let applications = Self::read_applications();

        Ok(ApplicationPlugin {
            applications,
            matcher,
        })
    }

    fn read_applications() -> Vec<AppResult> {
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
            .collect()
    }
}

impl Plugin<AppResult, AppMsg> for ApplicationPlugin {
    fn handle_msg(&mut self, _msg: AppMsg) {}

    fn refresh_content(&mut self) {
        let mut applications = Self::read_applications();
        self.applications.clear();
        self.applications.append(&mut applications);
    }

    fn handle_input(
        &self,
        user_input: &UserInput,
        history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<AppResult>> {
        let history_map = match history {
            Some(map) => {
                let res = map
                    .iter()
                    .map(|data| (data.id.clone(), data.score))
                    .collect::<HashMap<String, i32>>();

                Some(res)
            }
            None => None,
        };

        let result = self
            .applications
            .iter()
            .filter_map(|app| {
                let mut app = app.clone();
                match history_map.as_ref() {
                    Some(map) => {
                        if let Some(score) = map.get(app.get_id()) {
                            app.score = score.clone();
                            return Some(app);
                        }
                    }
                    None => {}
                }

                if user_input.input.is_empty() {
                    return Some(app);
                }

                let score = self
                    .matcher
                    .fuzzy_match(&app.app_name, &user_input.input)
                    .unwrap_or(0);

                if score > 0 {
                    app.score = score_utils::high(score);
                    Some(app)
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
