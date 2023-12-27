use self::mdx_utils::MDictLookup;
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::util::{score_utils, string_utils};
use tracing::info;

mod mdict;
mod mdx_utils;

pub const TYPE_ID: &str = "dict";

pub enum DictMsg {}

pub struct DictResult {
    pub word: String,
    pub html: String,
    pub dict: String,
}

impl PluginResult for DictResult {
    fn score(&self) -> i32 {
        return score_utils::middle(0);
    }

    fn sidebar_icon_name(&self) -> String {
        "dictionary".to_string()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.word.to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(string_utils::truncate(self.dict.as_str(), 60).to_string())
    }

    fn on_enter(&self) {}

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}

pub struct DictionaryPlugin {
    dir_path: String,
    mdxes: Vec<mdx_utils::MDictMemIndex>,
}

impl DictionaryPlugin {
    pub fn new(dir_path: &str) -> Self {
        let filepaths = crate::util::fs_utils::walk_dir(dir_path, Some(|p: &str| {
            p.to_lowercase().as_str().ends_with("mdx")
        }));
        
        
        let mdxes: Vec<mdx_utils::MDictMemIndex> = match filepaths {
            Ok(paths) => paths
                .into_iter()
                .filter_map(|dr| {
                        let p = dr.path();

                        match mdx_utils::MDictMemIndex::new(p) {
                            Ok(mdx) => Some(mdx),
                            Err(_) => None,
                        }
                    })
                .collect(),
            Err(_) => {
                vec![]
            }
        };

        info!("Creating Dict Plugin");
        DictionaryPlugin {
            dir_path: dir_path.to_string(),
            mdxes,
        }
    }

    pub fn seek(&self, word: &str) -> Vec<DictResult> {
        self.mdxes
            .iter()
            .filter_map(|mdx| {
                if let Ok(explain) = mdx.lookup_word(word) {
                    Some(DictResult {
                        word: word.to_string(),
                        html: explain,
                        dict: mdx.name.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn cycle_seek(&self, word: &str) -> Vec<DictResult> {
        let w = word.trim();
        let seek_res = self.seek(w);

        let mut res: Vec<DictResult> = vec![];
        for item in seek_res {
            if item.html.starts_with("@@@LINK=") {
                let w2 = item.html.replace("\r\n\0", "").replace("@@@LINK=", "");
                let r = self.cycle_seek(w2.as_str());
                res.extend(r);
            } else {
                res.push(item);
            }
        }

        res
    }
}

impl Plugin<DictResult, DictMsg> for DictionaryPlugin {
    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<DictResult>> {
        if user_input.input.is_empty() {
            return anyhow::Ok(vec![]);
        }

        anyhow::Ok(self.cycle_seek(user_input.input.as_str()))
    }

    fn handle_msg(&mut self, msg: DictMsg) {
        todo!()
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
