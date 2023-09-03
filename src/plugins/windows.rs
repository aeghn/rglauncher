use fragile::Fragile;
use std::process::Command;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use glib::Cast;
use gtk::Image;

use gtk::pango::WrapMode::{Word, WordChar};
use gtk::prelude::{GridExt, WidgetExt};
use gtk::Align::{Center};
use gtk::Grid;
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::plugins::{Plugin, PluginResult};
use crate::user_input::UserInput;

use gtk::Label;
use crate::icon_cache;

pub struct HyprWindows {
    windows: Vec<HyprWindowResult>,
}

struct Workspace {
    pub id: i64,
    pub name: String,
}

pub struct HyprWindowResult {
    pub class: String,
    pub title: String,
    pub icon_name: String,
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub pid: i64,
    pub xwayland: bool,
    pub monitor: i64,
    pub workspace: String,
    pub score: i32,
}

lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<(Grid, Image, Label, Label)>>> = Mutex::new(None);
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
            Ok(x) => x,
            Err(_) => serde_json::Value::Null,
        };

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
                    icon_name: Self::get_icon_name(class),
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

        HyprWindows { windows: vec }
    }

    fn get_icon_name(class: &str) -> String {
        let c = class;
        if class == "jetbrains-studio" {
            "android-studio".to_string()
        } else {
            c.to_string()
        }
    }
}

impl Clone for HyprWindowResult {
    fn clone(&self) -> Self {
        HyprWindowResult {
            class: self.class.clone(),
            title: self.title.clone(),
            icon_name: self.icon_name.to_string(),
            address: self.address.clone(),
            mapped: self.mapped.clone(),
            hidden: self.hidden.clone(),
            pid: self.pid.clone(),
            xwayland: self.xwayland.clone(),
            monitor: self.monitor.clone(),
            workspace: self.workspace.clone(),
            score: 0,
        }
    }
}

impl Plugin<HyprWindowResult> for HyprWindows {
    fn handle_input(&self, user_input: &UserInput) -> Vec<HyprWindowResult> {
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

        result
    }
}

impl PluginResult for HyprWindowResult {
    fn get_score(&self) -> i32 {
        return self.score;
    }

    fn sidebar_icon_name(&self) -> String {
        return self.icon_name.to_string();
    }

    fn sidebar_label(&self) -> Option<String> {
        let mut title = self.title.to_string();
        title.insert(0, ' ');
        title.insert(0, '');
        Some(title)
    }

    fn sidebar_content(&self) -> Option<String> {
        let str: String = format!(
            "{}  {} {}",
            if self.monitor == 0 {
                "".to_string()
            } else {
                format!(" {}", self.monitor)
            },
            self.workspace.clone(),
            if self.xwayland { "X" } else { "" }
        );

        Some(str)
    }

    fn preview(&self) -> gtk::Widget {
        let mut guard = PREVIEW.lock().unwrap();

        let wv = guard
            .get_or_insert_with(|| {
                let preview = Grid::builder()
                    .vexpand(true)
                    .hexpand(true)
                    .valign(Center)
                    .halign(Center)
                    .build();

                let big_pic = Image::builder()
                    .icon_name("gnome-windows")
                    .pixel_size(256)
                    .build();

                let little_pic = gtk::Image::builder()
                    .pixel_size(64)
                    .hexpand(true)
                    .build();

                preview.attach(&big_pic, 0, 0, 1, 1);
                preview.attach(&little_pic, 0, 1, 1, 1);

                let title = gtk::Label::builder()
                    .css_classes(["font16"])
                    .wrap(true)
                    .wrap_mode(WordChar)
                    .build();
                preview.attach(&title, 0, 2, 1, 1);

                let extra = gtk::Label::builder()
                    .wrap(true)
                    .wrap_mode(Word)
                    .hexpand(true)
                    .build();
                preview.attach(&extra, 0, 3, 1, 1);

                Fragile::new((preview, little_pic, title, extra))
            })
            .get();
        let (preview, image, title, extra) = wv;
        title.set_text(self.title.as_str());

        if let Some(c) = self.sidebar_content() {
            extra.set_text(c.as_str());
        }

        image.set_from_gicon(icon_cache::get_icon(self.icon_name.as_str()).get());

        preview.clone().upcast()
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
}
