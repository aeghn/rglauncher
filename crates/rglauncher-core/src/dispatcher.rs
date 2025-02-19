use crate::config::Config;
use crate::plugins::app::AppPlugin;
#[cfg(feature = "calc")]
use crate::plugins::calc::CalcPlugin;
#[cfg(feature = "clip")]
use crate::plugins::clip::{ClipPlugin, ClipReq};
use crate::plugins::history::{HistoryItem, HistoryPlugin};
#[cfg(feature = "mdict")]
use crate::plugins::mdict::{DictMsg, DictPlugin};
#[cfg(feature = "wmwin")]
use crate::plugins::win::WinPlugin;
use crate::plugins::{history, PRWrapper, Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::ResultMsg;
use arc_swap::ArcSwapOption;
use chin_tools::{AResult, EResult};
use chrono::NaiveDateTime;
use flume::Sender;
use futures::executor::ThreadPool;
use futures::task::SpawnExt;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

lazy_static! {
    static ref CONFIG: ArcSwapOption<Config> = ArcSwapOption::empty();
}

thread_local! {
    pub static CONNECTION: RefCell<Option<Connection>> = RefCell::new(None);
}

pub fn db_init() {
    CONNECTION.with_borrow_mut(|con| {
        let conn = Connection::open(&CONFIG.load().as_ref().unwrap().db.db_path).unwrap();

        let history = history::HistoryPlugin::new(Some(&conn));
        history.try_create_table().unwrap();

        con.replace(conn);
    })
}

#[derive(Clone)]
pub enum DispatchMsg {
    UserInput(Arc<UserInput>, Sender<ResultMsg>),
    RefreshContent,
    SetHistory(PRWrapper),
    PluginMsg,
}

pub struct PluginDispatcher {
    app: Arc<AppPlugin>,
    win: Arc<WinPlugin>,
    calc: Arc<CalcPlugin>,
    #[cfg(feature = "clip")]
    clip: Arc<ClipPlugin>,
    #[cfg(feature = "fmdict")]
    dict: Arc<DictPlugin>,
}

macro_rules! handle_input {
    ($user_input_arc:expr, $plugin:expr, $executor:expr, $sender:expr ) => {{
        let user_input = $user_input_arc.clone();
        let sender = $sender.clone();
        let plugin = $plugin.clone();
        if let Err(err) = $executor.spawn(async move {
            match plugin.handle_input(&user_input) {
                Ok(result) => {
                    if user_input.cancelled() {
                        tracing::info!("cancelled");
                        return;
                    }
                    sender
                        .send_async(ResultMsg::Result(
                            user_input,
                            result.into_iter().map(|e| e.into()).collect(),
                        ))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    tracing::error!(
                        "unable to handle input: {} -- {}",
                        plugin.get_type_id(),
                        err
                    );
                }
            }
        }) {
            tracing::error!("unable to spawn: {}", err);
        }
    }};
}

macro_rules! handle_refresh {
    ($executor:tt, $plugin:expr) => {
        let plugin = $plugin.clone();
        if let Err(err) = $executor.spawn(async move { plugin.refresh_content() }) {
            tracing::error!("unable to refresh {}", err)
        }
    };
}

impl PluginDispatcher {
    fn new(config: &Arc<Config>) -> AResult<PluginDispatcher> {
        let app = AppPlugin::new()?.into();
        let win = WinPlugin::new()?.into();
        #[cfg(feature = "clip")]
        let clip = ClipPlugin::new()?.into();
        #[cfg(feature = "fmdict")]
        let dict = DictPlugin::new(config.dict.as_ref())?.into();
        let calc = CalcPlugin::new()?.into();

        Ok(PluginDispatcher {
            app,
            win,
            #[cfg(feature = "clip")]
            clip,
            calc,
            #[cfg(feature = "fmdict")]
            dict,
        })
    }

    pub async fn spawn_blocking(config: &Arc<Config>, rx: flume::Receiver<DispatchMsg>) -> EResult {
        CONFIG.store(Some(config.clone()));

        let mut histories: HashMap<String, HistoryItem<crate::plugins::PluginResultEnum>> =
            HashMap::new();

        let dispatcher_arc = Arc::new(PluginDispatcher::new(&config)?);
        db_init();

        CONNECTION.with_borrow(|e| {
            if let Ok(historie_recs) = HistoryPlugin::new(e.as_ref()).fetch_histories() {
                histories.extend(historie_recs.into_iter().map(|h| (h.id.clone(), h)));
            }
        });

        let executor = ThreadPool::builder()
            .after_start(move |_| {
                db_init();
            })
            .name_prefix("rgl")
            .create()?;

        loop {
            match rx.recv_async().await? {
                DispatchMsg::UserInput(user_input, sender) => {
                    if user_input.input.trim().is_empty() {
                        let result = histories
                            .values()
                            .into_iter()
                            .map(|e| (e.body.clone(), e.weight as i32).into())
                            .collect();
                        sender
                            .send(ResultMsg::Result(user_input, result))
                            .expect("unable to send back history");
                        info!("send back history {}", histories.len());
                        continue;
                    }

                    let user_input_arc: Arc<UserInput> = user_input;

                    handle_input!(user_input_arc, dispatcher_arc.app, executor, sender);
                    handle_input!(user_input_arc, dispatcher_arc.win, executor, sender);
                    handle_input!(user_input_arc, dispatcher_arc.calc, executor, sender);
                    #[cfg(feature = "clip")]
                    handle_input!(user_input_arc, dispatcher_arc.clip, executor, sender);
                    #[cfg(feature = "fmdict")]
                    handle_input!(user_input_arc, dispatcher_arc.dict, executor, sender);
                }
                DispatchMsg::RefreshContent => {
                    handle_refresh!(executor, dispatcher_arc.app);
                    handle_refresh!(executor, dispatcher_arc.win);
                    handle_refresh!(executor, dispatcher_arc.calc);

                    #[cfg(feature = "clip")]
                    {
                        handle_refresh!(dispatcher_arc.clip);
                    }

                    #[cfg(feature = "fmdict")]
                    {
                        handle_refresh!(dispatcher_arc.dict);
                    }
                }
                DispatchMsg::SetHistory(prwrapper) => {
                    let history_id = HistoryPlugin::get_id(&prwrapper.body);
                    let history = histories.remove(&history_id);
                    let history = if let Some(history) = history {
                        HistoryItem {
                            weight: history.weight + 1.,
                            update_time: NaiveDateTime::default(),
                            ..history
                        }
                    } else {
                        HistoryItem {
                            id: history_id.clone(),
                            plugin_type: prwrapper.body.get_type_id().into(),
                            body: prwrapper.body,
                            weight: 1.,
                            update_time: NaiveDateTime::default(),
                        }
                    };

                    {
                        let history = history.clone();
                        CONNECTION.with_borrow(|e| {
                            let history_plg = HistoryPlugin::new(e.as_ref());
                            if let Err(err) = history_plg.update_or_insert(&history) {
                                error!("unable to insert history, {}", err);
                            }
                        })
                    }
                    histories.insert(history_id, history);
                }
                DispatchMsg::PluginMsg => {}
            }
        }
    }
}
