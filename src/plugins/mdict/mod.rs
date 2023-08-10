use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process::id;
use std::sync::Arc;
use futures::StreamExt;
use gio::Icon;
use glib::{Cast, StrV};
use gtk::AccessibleRole::Label;
use gtk::{Align, Grid, Widget};
use gtk::traits::{GridExt, StyleContextExt, WidgetExt};
use regex::Regex;
use rusqlite::Connection;
use tracing::error;
use webkit6::traits::WebViewExt;
use webkit6::{UserContentManager, UserStyleSheet, WebView};
use crate::plugins::mdict::mdict::{MDictIndex, MDictMode, MDictRecordBlockIndex, MDictRecordIndex};
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use crate::util::string_utils;

mod mdict;

pub struct Index {
    word: String,
    block: MDictRecordBlockIndex,
    idx: MDictRecordIndex
}

type DirType = String;
type MdxPathType = String;

pub struct MDictPlugin {
    pub(crate) conn: Option<Connection>,
    map: HashMap<MdxPathType, DirType>,
    re: Regex,
}

pub struct MDictPluginResult {
    word: String,
    html: String,
}

impl MDictPlugin {
    pub fn new(db_path: &str,
               files: Vec<MdxPathType>) -> Self {
        let conn = match Connection::open(db_path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        let mut map: HashMap<MdxPathType, DirType> = HashMap::new();
        for mpt in files {
            let file = File::open(mpt.as_str());
            match file {
                Ok(f) => {
                    match std::path::PathBuf::from(mpt.as_str()).parent() {
                        None => {}
                        Some(parent) => {
                            map.insert(mpt, parent.to_str().unwrap().to_string());
                        }
                    }

                }
                Err(_) => {}
            }
        }

        let webview = WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);

        let style_provider = gtk::CssProvider::new();
        // style_provider.load_from_data(include_str!("../../../resources/dict.css"));

        let context = webview.style_context();
        context.add_provider(&style_provider, 1);

        MDictPlugin {
            conn,
            map: Default::default(),
            re: Regex::new(r#"<link.*?/>|<script.*?/script>"#).unwrap(),
        }
    }

    pub fn seek(&self, word: &str) -> Vec<(String, String)>{
        let sql = "select word, explain, file_path from mdx_index where word = ?";
        if self.conn.is_none() {
            return vec![];
        }

        if let Some(_conn) = &self.conn {
            match _conn.prepare(sql) {
                Ok(mut e) => {
                    let iter = e.query_map(&[word], |row| {
                        let word = row.get(0).unwrap();
                        let explaination = row.get(1).unwrap();

                        Ok((word, explaination))
                    });
                    let mut vec = vec![];
                    if let Ok(_iter) = iter {
                        for cpr in _iter {
                            vec.push(cpr.unwrap());
                        }
                    }
                    vec
                }
                Err(_) => { vec![] }
            }
        } else {
            vec![]
        }
    }

    fn cycle_seek(&self, word: &str) -> Vec<(String, String)> {
        let w = word.trim();
        let seek_res = self.seek(w);

        let mut res: Vec<(String, String)> = vec![];
        for (word, explanation) in seek_res {
            let explanation = self.re.replace_all(explanation.as_str(), "").to_string();
            error!("{:?}", explanation);
            if explanation.starts_with("@@@LINK=") {
                let w2 = explanation.replace("\r\n\0", "").replace("@@@LINK=", "");
                let r = self.cycle_seek(w2.as_str());
                res.extend(r);
            } else {
                res.push((word, explanation))
            }
        }

        res
    }
}

impl Plugin<MDictPluginResult> for MDictPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<MDictPluginResult> {
        let res = self.cycle_seek(user_input.input.as_str());
        res.into_iter().map(|(word, explanation)| {
            MDictPluginResult{ word,
                html: explanation.replace("\0", "")
            }
        }).collect()
    }
}

impl PluginResult for MDictPluginResult {
    fn get_score(&self) -> i32 {
        return 100
    }

    fn sidebar_icon(&self) -> Option<Icon> {
        Some(gio::Icon::from(gio::ThemedIcon::from_names(&[
            &"dictionary",
        ])))
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.word.to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(string_utils::truncate(self.html.as_str(), 60).to_string())
    }

    fn preview(&self) -> Widget {
        let webview = WebView::with();
        webview.set_vexpand(true);
        webview.set_hexpand(true);

        let style_provider = gtk::CssProvider::new();
        style_provider.load_from_data(include_str!("../../../resources/dict.css"));

        let context = webview.style_context();
        context.add_provider(&style_provider, 1);

        // Load HTML content
        let mut html_content = self.html.replace("\0", " ");
        webview.load_html(html_content.as_str(), None);

        webview.upcast()

        // let grid = Grid::builder()
        //     .hexpand(true)
        //     .vexpand(true)
        //     .valign(Align::Center)
        //     .halign(Align::Center)
        //     .css_classes(StrV::from(vec!["borr"]))
        //     .build();
        //
        // grid.attach(&webview, 0, 0, 1, 1);
        //
        // grid.upcast()
    }

    fn on_enter(&self) {
        todo!()
    }
}