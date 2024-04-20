mod worker;

use crate::config::Config;
use crate::plugins::application::{AppMsg, ApplicationPlugin};
#[cfg(feature = "calc")]
use crate::plugins::calculator::{CalcMsg, CalculatorPlugin};
#[cfg(feature = "clip")]
use crate::plugins::clipboard::{ClipMsg, ClipboardPlugin};
#[cfg(feature = "mdict")]
use crate::plugins::dictionary::{DictMsg, DictionaryPlugin};
use crate::plugins::history::{HistoryItem, HistoryPlugin};
#[cfg(feature = "hyprwin")]
use crate::plugins::windows::{HyprWindowMsg, HyprWindowsPlugin};
use crate::plugins::PluginResult;
use crate::userinput::UserInput;
use crate::ResultMsg;
use flume::Sender;
use futures::executor::block_on;
use std::sync::{Arc, RwLock};
use tracing::info;

use self::worker::PluginWorker;

pub enum PluginMsg {
    App(AppMsg),
    #[cfg(feature = "calc")]
    Calc(CalcMsg),
    #[cfg(feature = "clip")]
    Clip(ClipMsg),
    #[cfg(feature = "mdict")]
    Dict(DictMsg),
    #[cfg(feature = "hyprwin")]
    Hypr(HyprWindowMsg),
}

#[derive(Clone)]
pub enum DispatchMsg {
    UserInput(Arc<UserInput>, Sender<ResultMsg>),
    RefreshContent,
    SetHistory(Arc<dyn PluginResult>),
    PluginMsg,
}

pub struct PluginDispatcher {
    history: HistoryPlugin,
    inner: InnerDispatcher,
}

#[derive(Clone)]
struct InnerDispatcher {
    dispatch_rx: async_broadcast::Receiver<DispatchMsg>,
    history: Arc<RwLock<Vec<HistoryItem>>>,
}

fn to_option<T>(r: anyhow::Result<T>) -> Option<T> {
    r.map_or(None, |e| Some(e))
}

impl PluginDispatcher {
    fn new(
        config: &Arc<Config>,
        dispatch_rx: async_broadcast::Receiver<DispatchMsg>,
    ) -> PluginDispatcher {
        let history = HistoryPlugin::new(config.db.as_ref());

        let inner = InnerDispatcher {
            dispatch_rx: dispatch_rx,
            history: history.get_cache(),
        };

        PluginWorker::launch(|| ApplicationPlugin::new(), &inner, "app");

        #[cfg(feature = "hyprwin")]
        PluginWorker::launch(|| HyprWindowsPlugin::new(), &inner, "hyprwin");

        #[cfg(feature = "clip")]
        {
            let db_config = config.db.clone();
            PluginWorker::launch(
                move || ClipboardPlugin::new(db_config.as_ref()),
                &inner,
                "clip",
            );
        }

        #[cfg(feature = "mdict")]
        {
            let dict_config = config.dict.clone();
            PluginWorker::launch(
                move || DictionaryPlugin::new(dict_config.as_ref()),
                &inner,
                "mdict",
            );
        }

        #[cfg(feature = "calc")]
        PluginWorker::launch(|| CalculatorPlugin::new(), &inner, "calc");

        PluginDispatcher { history, inner }
    }

    async fn forward(&self) {
        let mut dispatcher_rx = self.inner.dispatch_rx.clone();

        loop {
            if let Ok(DispatchMsg::SetHistory(msg)) = dispatcher_rx.recv().await {
                self.history
                    .update_or_insert(msg)
                    .expect("unable to insert history");
            }
        }
    }

    pub fn start(config: &Arc<Config>) -> async_broadcast::Sender<DispatchMsg> {
        let (mut dispatch_tx, dispatch_rx) = async_broadcast::broadcast(30);
        dispatch_tx.set_overflow(true);

        let config = config.clone();
        std::thread::Builder::new()
            .name("dispatcher".to_string())
            .spawn(move || {
                block_on(async {
                    let dispatcher = PluginDispatcher::new(&config, dispatch_rx);
                    dispatcher.forward().await
                })
            })
            .unwrap();

        dispatch_tx
    }
}
