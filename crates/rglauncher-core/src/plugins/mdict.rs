use fuzzy_matcher::skim::SkimMatcherV2;
use mdict::mdx_utils::{self, MDictLookup};

use crate::config::DictConfig;
use crate::plugins::{PluginItemTrait, PluginTrait};
use crate::userinput::UserInput;
use crate::util::scoreutils;

pub const TYPE_NAME: &str = "mdict";

#[derive(Clone)]
pub enum DictMsg {}

#[derive(Clone)]
pub struct MDictItem {
    pub word: String,
    pub html: String,
    pub dict: String,
    id: String,
    pub score: i32,
}

impl PluginItemTrait for MDictItem {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn on_activate(&self) {}

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub struct MDictPlugin {
    mdxes: Vec<mdx_utils::MDictMemIndex>,
    matcher: SkimMatcherV2,
}

impl MDictPlugin {
    pub fn new(dict_config: Option<&DictConfig>) -> anyhow::Result<Self> {
        match dict_config.map(|e| e.dir_path.as_str()) {
            Some(dir) => {
                let filepaths = crate::util::fileutils::walk_dir(
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
                        Ok(MDictPlugin {
                            mdxes,
                            matcher: SkimMatcherV2::default(),
                        })
                    }
                    Err(err) => anyhow::bail!(err),
                }
            }
            None => anyhow::bail!("missing dict config!"),
        }
    }

    pub fn seek(&self, word: &str) -> Vec<MDictItem> {
        self.mdxes
            .iter()
            .filter_map(|mdx| {
                if let Ok(explain) = mdx.lookup_word(word) {
                    Some(MDictItem {
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

    fn cycle_seek(&self, word: &str) -> Vec<MDictItem> {
        let w = word.trim();
        let seek_res = self.seek(w);

        let mut res: Vec<MDictItem> = vec![];
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

impl PluginTrait for MDictPlugin {
    type Msg = DictMsg;

    type Item = MDictItem;

    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<MDictItem>> {
        let mut result = vec![];
        if !user_input.input.is_empty() {
            let mut res = self.cycle_seek(user_input.input.as_str());
            res.iter_mut().for_each(|r| {
                r.score = scoreutils::middle(r.score.clone() as i64);
            });
            result.extend(res);
        }

        Ok(result)
    }

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }
}
