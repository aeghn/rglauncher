use crate::icon_cache;
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use fragile::Fragile;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gio::prelude::AppInfoExt;
use glib::{Cast};
use gtk::prelude::{ButtonExt, GridExt, WidgetExt};
use lazy_static::lazy_static;
use std::option::Option::None;

use std::sync::Mutex;
use crate::util::score_utils;


lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<(gtk::Widget, gtk::Image, gtk::Label, gtk::Label, gtk::Label)>>> =
        Mutex::new(None);
}

pub struct AppPlugin {}

pub struct AppResult {
    icon_name: String,
    app_name: String,
    app_desc: String,
    score: i32,
    pub id: String,
}

impl PluginResult for AppResult {
    fn get_score(&self) -> i32 {
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

    fn preview(&self) -> gtk::Widget {
        let mut guard = PREVIEW.lock().unwrap();

        let wv = guard
            .get_or_insert_with(|| {
                let preview = gtk::Grid::builder()
                    .vexpand(true)
                    .hexpand(true)
                    .valign(gtk::Align::Center)
                    .halign(gtk::Align::Center)
                    .build();

                let icon = gtk::Image::builder().pixel_size(256).build();
                preview.attach(&icon, 0, 0, 1, 1);

                let name = gtk::Label::builder().css_classes(["font32"]).wrap(true).build();
                preview.attach(&name, 0, 1, 1, 1);

                let desc = gtk::Label::builder().wrap(true).build();
                preview.attach(&desc, 0, 2, 1, 1);

                let exec = gtk::Label::builder().wrap(true).build();
                preview.attach(&exec, 0, 3, 1, 1);

                Fragile::new((preview.upcast(), icon, name, exec, desc))
            })
            .get();

        let (preview, image, name, exec, desc) = wv;
        image.set_from_gicon(icon_cache::get_icon(self.app_name.as_str()).get());
        name.set_label(self.app_name.as_str());
        exec.set_label("");
        desc.set_label(self.app_desc.as_str());

        preview.clone()
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

        gio::AppInfo::all().iter().for_each(|app_info| {
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
                        None => "".to_string(),
                        Some(des) => des.to_string(),
                    },
                    score,
                });
            }
        });

        result
    }
}
