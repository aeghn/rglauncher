use std::process::Command;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use gio::Icon;
use glib::Cast;
use gtk::{Label, Widget};

use gtk::pango::WrapMode::WordChar;
use gtk::prelude::{GridExt, WidgetExt};
use tracing::{error};

use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;

pub struct HyprWindows {
    windows: Vec<HyprWindowResult>
}

struct Workspace {
    pub id: i64,
    pub name: String,
}

pub struct HyprWindowResult {
    pub class: String,
    pub title: String,
    pub icon: Option<gio::Icon>,
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub pid: i64,
    pub xwayland: bool,
    pub monitor: i64,
    pub workspace: String,
    pub score: i32,
}

impl HyprWindows {
    pub fn new() -> Self {
        let output = Command::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .unwrap();
        let mut vec: Vec<HyprWindowResult> = vec![];

        let out = String::from_utf8(output.stdout).unwrap();
        // let json : serde_json::Value =

        let json = match serde_json::from_str::<serde_json::Value>(out.as_str()) {
            Ok(x) => {
                x
            }
            Err(_) => {
                serde_json::Value::Null
            }
        };

        if let Some(array) = json.as_array() {
            for e in array {
                let class = e.get("class").unwrap().as_str().unwrap();
                vec.push(HyprWindowResult {
                    class: class.to_string(),
                    title: e.get("title").unwrap().as_str().unwrap().to_string(),
                    icon: gio::Icon::for_string(class).ok(),
                    address: e.get("address").unwrap().as_str().unwrap().to_string(),
                    mapped: e.get("mapped").unwrap().as_bool().unwrap(),
                    hidden: e.get("hidden").unwrap().as_bool().unwrap(),
                    pid: e.get("pid").unwrap().as_i64().unwrap(),
                    xwayland: e.get("xwayland").unwrap().as_bool().unwrap(),
                    monitor: e.get("monitor").unwrap().as_i64().unwrap(),
                    workspace: e.get("workspace").unwrap().get("name").unwrap()
                        .as_str().unwrap().to_string(),
                    score: 0
                })
            }
        }

        HyprWindows {
            windows: vec
        }
    }
}

impl Clone for HyprWindowResult {
    fn clone(&self) -> Self {
        HyprWindowResult {
            class: self.class.clone(),
            title: self.title.clone(),
            icon: self.icon.clone(),
            address: self.address.clone(),
            mapped: self.mapped.clone(),
            hidden: self.hidden.clone(),
            pid: self.pid.clone(),
            xwayland: self.xwayland.clone(),
            monitor: self.monitor.clone(),
            workspace: self.workspace.clone(),
            score: 0
        }
    }
}

impl Plugin for HyprWindows {
    fn handle_input(&self, user_input: &UserInput) -> Vec<Box<dyn PluginResult>> {
        let matcher = SkimMatcherV2::default();
        let mut result: Vec<Box<dyn PluginResult>> = vec![];

        for window in &self.windows {
            let mut score : i32= 0;
            let mut mstr = window.class.to_string();
            mstr += window.title.as_str();
            if user_input.input.is_empty() {
                score = 100;
            } else if let Some(_s) = matcher.fuzzy_match(mstr.as_str(), user_input.input.as_str()) {
                score = _s as i32;
            }

            let mut mw = window.clone();
            mw.score = score;
            if score > 0 {
                result.push(Box::new(mw));
            }
        }

        result
    }
}

impl PluginResult for HyprWindowResult {
    fn get_score(&self) -> i32 {
        return self.score;
    }

    fn sidebar_icon(&self) -> Option<Icon> {
        if let Some(icon) = &self.icon {
            Some(icon.clone())
        } else {
            Some(gio::Icon::from(gio::ThemedIcon::from_names(&[&"gnome-windows"])))
        }
    }

    fn sidebar_label(&self) -> Option<String> {
        let mut title = self.title.to_string();
        title.insert(0, ' ');
        title.insert(0, '');
        Some(title)
    }

    fn sidebar_content(&self) -> Option<Widget> {
        let str : String = format!("{}  {} {}",
                                   if self.monitor == 0 {"".to_string()} else {format!(" {}", self.monitor)},
                                   self.workspace.clone(),
                                   if self.xwayland {"XWayland"} else {""});
        let label = Label::new(Some(str.as_str()));
        label.set_wrap(true);
        Some(label.upcast())
    }

    fn preview(&self) -> gtk::Grid {
        let preview = gtk::Grid::new();

        preview.set_hexpand(true);
        preview.set_vexpand(true);

        let image = gtk::Image::from_icon_name("gnome-windows");
        image.set_pixel_size(256);
        preview.attach(&image, 0, 0, 2, 1);

        let image = self.sidebar_icon().unwrap();
        let image = gtk::Image::from_gicon(&image);
        image.set_pixel_size(64);
        preview.attach(&image, 0, 1, 1, 2);

        let name = gtk::Label::new(Some(self.sidebar_label().unwrap().as_str()));
        name.add_css_class("font16");
        name.set_wrap(true);
        name.set_wrap_mode(WordChar);
        preview.attach(&name, 1, 1, 1, 1);

        if let Some(content) = self.sidebar_content() {
            preview.attach(&content, 1, 2, 1, 1);
        }

        preview
    }

    fn on_enter(&self) {
        // dispatch focuswindow address:
        let msg = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg("address:".to_owned() + self.address.as_str())
            .output()
            .expect("unable to switch to the window");
        error!("msg: {:?}", msg);
    }
}