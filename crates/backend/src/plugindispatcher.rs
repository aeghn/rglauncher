use crate::plugins::app::{AppMsg, ApplicationPlugin};
use crate::plugins::calculator::{CalcMsg, CalculatorPlugin};
use crate::plugins::clipboard::{ClipMsg, ClipboardPlugin};
use crate::plugins::dict::{DictMsg, DictionaryPlugin};
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
    pub dispatch_sender: Sender<DispatchMsg>,
    dispatcher_receiver: Receiver<DispatchMsg>,

    wind_sender: Sender<PluginMsg<HyprWindowMsg>>,
    app_sender: Sender<PluginMsg<AppMsg>>,
    clip_sender: Sender<PluginMsg<ClipMsg>>,
    dict_sender: Sender<PluginMsg<DictMsg>>,
    calc_sender: Sender<PluginMsg<CalcMsg>>,

    history: HistoryPlugin,
}

macro_rules! send_to_senders {
    ($msg:expr, $($sender:expr),*) => {
        $(
            $sender.send($msg.clone()).expect("TODO: panic message");
        )*
    };
}

impl PluginDispatcher {
    fn new(dictionary_dir: &str, clipboard_path: &str) -> PluginDispatcher {
        let (dispatcher_sender, dispatcher_receiver) = flume::unbounded();

        let history = HistoryPlugin::new(clipboard_path).unwrap();

        let dictionary_dir = dictionary_dir.to_string();
        let clipboard_path = clipboard_path.to_string();

        let wind_sender = PluginWorker::launch(|| HyprWindowsPlugin::new());
        let app_sender = PluginWorker::launch(|| ApplicationPlugin::new());
        let clip_sender =
            PluginWorker::launch(move || ClipboardPlugin::new(clipboard_path.as_str()));
        let dict_sender =
            PluginWorker::launch(move || DictionaryPlugin::new(dictionary_dir.as_str()));
        let calc_sender = PluginWorker::launch(|| CalculatorPlugin::new());

        PluginDispatcher {
            dispatch_sender: dispatcher_sender,
            dispatcher_receiver,

            wind_sender,
            app_sender,
            clip_sender,
            dict_sender,
            calc_sender,

            history,
        }
    }

    async fn forward(&self) {
        let dispatcher_receiver = self.dispatcher_receiver.clone();

        loop {
            match dispatcher_receiver.recv_async().await {
                Ok(dispatch_msg) => match dispatch_msg {
                    DispatchMsg::UserInput(user_input, result_sender) => {
                        let history = self.history.get_histories(&user_input);

                        let history = Arc::new(history);
                        
                        send_to_senders!(
                            PluginMsg::UserInput(user_input.clone(), result_sender.clone(), history.clone()),
                            self.app_sender,
                            self.wind_sender,
                            self.clip_sender,
                            self.dict_sender,
                            self.calc_sender
                        );
                    }
                    DispatchMsg::RefreshContent => {
                        send_to_senders!(
                            PluginMsg::RefreshContent,
                            self.app_sender,
                            self.wind_sender,
                            self.clip_sender,
                            self.dict_sender,
                            self.calc_sender
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

    pub fn start(dictionary_dir: &str, clipboard_path: &str) -> Sender<DispatchMsg> {
        let dispatcher = PluginDispatcher::new(dictionary_dir, clipboard_path);
        let dispatch_sender = dispatcher.dispatch_sender.clone();
        std::thread::spawn(move || block_on(dispatcher.forward()));

        dispatch_sender
    }
}
