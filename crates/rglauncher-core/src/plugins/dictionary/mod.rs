use self::mdx_utils::MDictLookup;
use crate::config::DictConfig;
use crate::plugins::history::HistoryItem;
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::util::score_utils;

#[allow(dead_code)]
mod mdict;
#[allow(dead_code)]
mod mdx_utils;

pub const TYPE_ID: &str = "dict";

#[derive(Clone)]
pub enum DictMsg {}

pub struct DictResult {
    pub word: String,
    pub html: String,
    pub dict: String,
    id: String,
    pub score: i32,
}

impl PluginResult for DictResult {
    fn score(&self) -> i32 {
        self.score
    }

    fn icon_name(&self) -> &str {
        "org.gnome.Dictionary"
    }

    fn name(&self) -> &str {
        self.word.as_str()
    }

    fn extra(&self) -> Option<&str> {
        Some(self.dict.as_str())
    }

    fn on_enter(&self) {}

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub struct DictionaryPlugin {
    mdxes: Vec<mdx_utils::MDictMemIndex>,
}

impl DictionaryPlugin {
    pub fn new(dict_config: Option<&DictConfig>) -> anyhow::Result<Self> {
        match dict_config.map(|e| e.dir_path.as_str()) {
            Some(dir) => {
                let filepaths = crate::util::fs_utils::walk_dir(
                    dir,
                    Some(|p: &str| p.to_lowercase().as_str().ends_with("mdx")),
                );

                match filepaths {
                    Ok(paths) => {
                        let mdxes = paths
                            .into_iter()
                            .filter_map(|dr| {
                                let p = dr.path();

                                match mdx_utils::MDictMemIndex::new(p) {
                                    Ok(mdx) => Some(mdx),
                                    Err(_) => None,
                                }
                            })
                            .collect();
                        Ok(DictionaryPlugin { mdxes })
                    }
                    Err(err) => anyhow::bail!(err),
                }
            }
            None => anyhow::bail!("2"),
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
                        id: format!("{}@{}", mdx.name.as_str(), word),
                        score: 0,
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
    fn handle_msg(&mut self, _msg: DictMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {}

    fn handle_input(
        &self,
        user_input: &UserInput,
        history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<DictResult>> {
        let mut result = vec![];
        if !user_input.input.is_empty() {
            let mut res = self.cycle_seek(user_input.input.as_str());
            res.iter_mut().for_each(|r| {
                r.score = score_utils::middle(r.score.clone() as i64);
            });
            result.extend(res);
        }

        let _history_matcher = match history {
            None => {}
            Some(his) => his.iter().for_each(|h| {
                let dict_and_word: Vec<&str> = h.id.split("@").collect();
                let temp: Vec<DictResult> = self
                    .cycle_seek(dict_and_word[1])
                    .into_iter()
                    .map(|mut r| {
                        r.score = h.score;
                        r
                    })
                    .collect();
                result.extend(temp);
            }),
        };

        Ok(result)
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
