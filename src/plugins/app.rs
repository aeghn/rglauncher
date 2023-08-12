use std::sync::Arc;
use fragile::Fragile;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gio::prelude::AppInfoExt;
use gio::AppInfo;
use glib::{Cast, GString, StrV};
use glib::subclass::types::FromObject;
use gtk::prelude::{GridExt, WidgetExt};
use gtk::Align::Center;
use lazy_static::lazy_static;
use std::option::Option::None;
use crate::icon_cache;

use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;

lazy_static!{
    static ref PREVIEW: Arc<Option<Fragile<gtk::Grid>>> = Arc::new(None);
}

pub struct AppPlugin {}

pub struct AppResult {
    icon_name: String,
    app_name: String,
    app_desc: String,
    executable: String,
    score: i32,
    pub id: String,
}

impl PluginResult for AppResult {
    fn get_score(&self) -> i32 {
        self.score
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

    fn preview(&self) -> gtk::Widget {
        let preview = gtk::Grid::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(Center)
            .halign(Center)
            .css_classes(StrV::from(["centercld"]))
            .build();

        let image = gtk::Image::from_gicon(icon_cache::get_icon(self.app_name.as_str()).get());
        image.set_pixel_size(256);
        preview.attach(&image, 0, 0, 1, 1);

        let name = gtk::Label::builder()
            .label(self.app_name.as_str())
            .css_classes(StrV::from(["font32"]))
            .wrap(true)
            .build();

        preview.attach(&name, 0, 1, 1, 1);

        let desc = gtk::Label::builder().label(self.app_desc.as_str()).wrap(true).build();
        preview.attach(&desc, 0, 2, 1, 1);

        preview.upcast()
    }

    fn on_enter(&self) {
        AppInfo::all().iter().for_each(|app_info| {
            if app_info.id().unwrap().to_string() == self.id {
                app_info
                    .launch(&[], gio::AppLaunchContext::NONE)
                    .expect("unable to start app");
            }
        });
    }
}

impl AppPlugin {
    pub fn new() -> Self {
        AppPlugin {}
    }
}

impl Plugin<AppResult> for AppPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<AppResult> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<AppResult> = vec![];

        AppInfo::all().iter().for_each(|app_info| {
            if !app_info.should_show() {
                return;
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
                result.push(AppResult {
                    id: app_info.id().unwrap().to_string(),
                    icon_name: app_info.name().to_string(),
                    app_name: app_info.name().to_string(),
                    app_desc: match app_info.description() {
                        None => {"".to_string()}
                        Some(des) => {des.to_string()}
                    },
                    score,
                    executable: app_info.executable().to_str().unwrap().to_string(),
                });
            }
        });

        result
    }
}
