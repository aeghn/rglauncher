use crate::plugins::{Plugin, PluginPreview, PluginResult};
use crate::userinput::UserInput;
use crate::{icon_cache};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gio::prelude::AppInfoExt;
use glib::Cast;
use gtk::prelude::{ButtonExt, GridExt, WidgetExt};
use std::option::Option::None;

use crate::util::score_utils;

pub enum AppMsg {

}

pub struct AppResult {
    icon_name: String,
    app_name: String,
    app_desc: String,
    score: i32,
    pub id: String,
}

pub const TYPE_ID : &str = "app_result";

impl PluginResult for AppResult {
    fn score(&self) -> i32 {
        score_utils::high(self.score as i64)
    }

    fn sidebar_icon_name(&self) -> String {
        self.app_name.clone()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.app_name.clone())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(self.app_desc.clone())
    }

    fn on_enter(&self) {
        gio::AppInfo::all().iter().for_each(|app_info| {
            if app_info.id().unwrap().to_string() == self.id {
                app_info
                    .launch(&[], gio::AppLaunchContext::NONE)
                    .expect("unable to start app");
            }
        });
    }

    fn get_type_id(&self) -> &'static str {
        TYPE_ID
    }
}

pub struct ApplicationPlugin {}

impl ApplicationPlugin {
    pub fn new() -> Self {
        ApplicationPlugin {}
    }
}

impl Plugin<AppResult, AppMsg> for ApplicationPlugin {
    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<AppResult>> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<AppResult> = gio::AppInfo::all()
            .iter()
            .filter_map(|app_info| {
                if !app_info.should_show() {
                    return None;
                }

                let mut score: i32 = 0;
                if user_input.input.is_empty() {
                    score = 100;
                } else if let Some(_s) =
                    matcher.fuzzy_match(app_info.name().as_str(), user_input.input.as_str())
                {
                    score = _s as i32;
                }

                if score > 0 {
                    Some(AppResult {
                        id: app_info.id().unwrap().to_string(),
                        icon_name: app_info.name().to_string(),
                        app_name: app_info.name().to_string(),
                        app_desc: match app_info.description() {
                            None => "".to_string(),
                            Some(des) => des.to_string(),
                        },
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }

    fn handle_msg(msg: AppMsg) {
        
    }
}

pub struct AppPreview {
    root: gtk::Grid,
    icon: gtk::Image,
    name: gtk::Label,
    desc: gtk::Label,
    exec: gtk::Label,
}

impl PluginPreview for AppPreview {
    type PluginResult = AppResult;
    fn new() -> Self {
        let preview = gtk::Grid::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        let icon = gtk::Image::builder().pixel_size(256).build();
        preview.attach(&icon, 0, 0, 1, 1);

        let name = gtk::Label::builder()
            .css_classes(["font32"])
            .wrap(true)
            .build();
        preview.attach(&name, 0, 1, 1, 1);

        let desc = gtk::Label::builder().wrap(true).build();
        preview.attach(&desc, 0, 2, 1, 1);

        let exec = gtk::Label::builder().wrap(true).build();
        preview.attach(&exec, 0, 3, 1, 1);

        AppPreview {
            root: preview,
            icon,
            name,
            desc,
            exec,
        }
    }

    fn get_preview(&self, plugin_result: &AppResult) -> gtk::Widget {
        self.icon
            .set_from_gicon(icon_cache::get_icon(plugin_result.app_name.as_str()).get());
        self.name.set_label(plugin_result.app_name.as_str());
        self.exec.set_label("");
        self.desc.set_label(plugin_result.app_desc.as_str());

        self.root.clone().upcast()
    }
}