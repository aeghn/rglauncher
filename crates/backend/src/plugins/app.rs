use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use freedesktop_desktop_entry::DesktopEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::option::Option::None;

use crate::util::score_utils;

pub enum AppMsg {}

#[derive(Debug, Clone)]
pub struct AppResult {
    icon_name: String,
    pub app_name: String,
    pub app_desc: String,
    pub exec_path: String,
    score: i32,
    pub id: String,
}

pub const TYPE_ID: &str = "app_result";

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

    fn on_enter(&self) {}

    fn get_type_id(&self) -> &'static str {
        TYPE_ID
    }
}

pub struct ApplicationPlugin {
    applications: Vec<AppResult>,
    matcher: SkimMatcherV2,
}

impl ApplicationPlugin {
    pub fn new() -> Self {
        let matcher = SkimMatcherV2::default();

        let applications =
            freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
                .into_iter()
                .filter_map(|path| {
                    if let Ok(bytes) = std::fs::read_to_string(&path) {
                        if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                            return Some(AppResult {
                                id: entry.id().to_string(),
                                icon_name: entry.icon().unwrap_or_default().to_string(),
                                app_name: entry.name(None).unwrap_or_default().to_string(),
                                app_desc: entry.comment(None).unwrap_or_default().to_string(),
                                exec_path: entry.exec().unwrap_or_default().to_string(),
                                score: 0,
                            });
                        }
                    }
                    None
                })
                .collect();

        ApplicationPlugin {applications, matcher}
    }
}

impl Plugin<AppResult, AppMsg> for ApplicationPlugin {
    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<AppResult>> {
        let result = self.applications
        .iter()
        .filter_map(|app| {
            let score = self.matcher.fuzzy_match(&app.app_name, &user_input.input);

            if score.unwrap_or(0) > 0 {
                Some(app.clone())
            } else {
                None
            }
        }).collect();

        Ok(result)
    }

    fn handle_msg(&mut self, msg: AppMsg) {
        
    }
}


#[cfg(test)]
mod tests {
    use crate::{plugins::{app::ApplicationPlugin, Plugin}, userinput::UserInput};

    #[test]
    fn test_app()    {
        let app_plugin = ApplicationPlugin::new();
        println!("apps: {:?}", app_plugin.handle_input(&UserInput::new("a")));
    }
}