use crate::config::Config;
use crate::plugins::application::{AppMsg, ApplicationPlugin};
use crate::plugins::calculator::{CalcMsg, CalculatorPlugin};
use crate::plugins::clipboard::{ClipMsg, ClipboardPlugin};
use crate::plugins::dictionary::{DictMsg, DictionaryPlugin};
use crate::plugins::history::HistoryPlugin;
use crate::plugins::windows::{HyprWindowMsg, HyprWindowsPlugin};
use crate::plugins::{PluginMsg, PluginResult};
use crate::userinput::UserInput;
use crate::ResultMsg;
use flume::{Receiver, Sender};
use futures::executor::block_on;
use std::sync::Arc;

use super::pluginworker::PluginWorker;

pub enum DispatchMsg {
    UserInput(Arc<UserInput>, Sender<ResultMsg>),
    RefreshContent,
    SetHistory(Arc<dyn PluginResult>),
}

pub struct PluginDispatcher {
    pub dispatch_tx: Sender<DispatchMsg>,
    dispatcher_rx: Receiver<DispatchMsg>,

    wind_tx: Sender<PluginMsg<HyprWindowMsg>>,
    app_tx: Sender<PluginMsg<AppMsg>>,
    clip_tx: Sender<PluginMsg<ClipMsg>>,
    dict_tx: Sender<PluginMsg<DictMsg>>,
    calc_tx: Sender<PluginMsg<CalcMsg>>,

    history: HistoryPlugin,
}

macro_rules! send_to_txs {
    ($msg:expr, $($sender:expr),*) => {
        $(
            $sender.send($msg.clone()).expect("TODO: panic message");
        )*
    };
}

impl PluginDispatcher {
    fn new(config: &Arc<Config>) -> PluginDispatcher {
        let (dispatcher_tx, dispatcher_rx) = flume::unbounded();

        let history = HistoryPlugin::new(&config.db.db_path).unwrap();

        let dictionary_dir = config.dict.dir_path.to_string();
        let clipboard_path = config.db.db_path.to_string();

        let wind_tx = PluginWorker::launch("windows", || HyprWindowsPlugin::new());
        let app_tx = PluginWorker::launch("apps", || ApplicationPlugin::new());
        let clip_tx = PluginWorker::launch("clips", move || {
            ClipboardPlugin::new(clipboard_path.as_str())
        });
        #[cfg(feature = "dict")]
        let dict_tx = PluginWorker::launch("dict", move || {
            DictionaryPlugin::new(dictionary_dir.as_str())
        });
        let calc_tx = PluginWorker::launch("calc", || CalculatorPlugin::new());

        PluginDispatcher {
            dispatch_tx: dispatcher_tx,
            dispatcher_rx,
            wind_tx,
            app_tx,
            clip_tx,
            dict_tx,
            calc_tx,

            history,
        }
    }

    async fn forward(&self) {
        let dispatcher_rx = self.dispatcher_rx.clone();

        loop {
            match dispatcher_rx.recv_async().await {
                Ok(dispatch_msg) => match dispatch_msg {
                    DispatchMsg::UserInput(user_input, result_tx) => {
                        let history = self.history.get_histories(&user_input);

                        let history = Arc::new(history);

                        send_to_txs!(
                            PluginMsg::UserInput(
                                user_input.clone(),
                                result_tx.clone(),
                                history.clone()
                            ),
                            self.app_tx,
                            self.wind_tx,
                            self.clip_tx,
                            self.dict_tx,
                            self.calc_tx
                        );
                    }
                    DispatchMsg::RefreshContent => {
                        send_to_txs!(
                            PluginMsg::RefreshContent,
                            self.app_tx,
                            self.wind_tx,
                            self.clip_tx,
                            self.dict_tx,
                            self.calc_tx
                        );
                    }
                    DispatchMsg::SetHistory(msg) => {
                        self.history
                            .update_or_insert(msg)
                            .expect("unable to insert history");
                    }
                },
                Err(_) => {}
            }
        }
    }

    pub fn start(config: &Arc<Config>) -> Sender<DispatchMsg> {
        let dispatcher = PluginDispatcher::new(&config);
        let dispatch_tx = dispatcher.dispatch_tx.clone();
        std::thread::spawn(move || block_on(dispatcher.forward()));

        dispatch_tx
    }
}
