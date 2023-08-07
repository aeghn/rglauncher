use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gio::prelude::AppInfoExt;
use gio::AppInfo;
use glib::Cast;
use gtk::prelude::{GridExt, WidgetExt};
use gtk::Widget;

use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;

pub struct AppPlugin {}

pub struct AppResult {
    app_info: AppInfo,
    score: i32,
}

unsafe impl Send for AppResult {}

impl PluginResult for AppResult {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn sidebar_icon(&self) -> Option<gio::Icon> {
        if let Some(icon) = self.app_info.icon() {
            Some(icon)
        } else {
            Some(gio::Icon::from(gio::ThemedIcon::from_names(&[
                &"gnome-windows",
            ])))
        }
    }

    fn sidebar_label(&self) -> Option<String> {
        let name = self.app_info.name().to_string();
        Some(name)
    }

    fn sidebar_content(&self) -> Option<Widget> {
        let label = gtk::Label::new(Some(
            match self.app_info.description() {
                Some(x) => x.to_string(),
                None => "".to_string(),
            }
            .as_str(),
        ));
        label.set_wrap_mode(gtk::pango::WrapMode::Word);
        label.set_wrap(true);
        Some(label.upcast())
    }

    fn preview(&self) -> gtk::Grid {
        let preview = gtk::Grid::new();
        preview.add_css_class(&"centercld");

        let image = if let Some(icon) = self.app_info.icon() {
            icon
        } else {
            gio::Icon::from(gio::ThemedIcon::from_names(&[&"gnome-windows"]))
        };
        let image = gtk::Image::from_gicon(&image);
        image.set_pixel_size(256);
        preview.attach(&image, 0, 0, 1, 1);

        let name = gtk::Label::new(Some(self.app_info.name().as_str()));
        name.add_css_class("font32");
        name.set_wrap(true);
        preview.attach(&name, 0, 1, 1, 1);

        if let Some(gdesc) = self.app_info.description() {
            let desc = gtk::Label::new(Some(gdesc.as_str()));
            desc.set_wrap(true);
            preview.attach(&desc, 0, 2, 1, 1);
        }

        preview
    }

    fn on_enter(&self) {
        self.app_info
            .launch(&[], gio::AppLaunchContext::NONE)
            .expect("unable to start app");
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
                    app_info: app_info.clone(),
                    score,
                });
            }
        });

        result
    }
}
