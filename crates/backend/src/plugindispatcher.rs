use std::sync::Arc;
use crate::plugins::ResultMsg;
use crate::plugins::app::ApplicationPlugin;
use crate::plugins::calculator::CalculatorPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::dict::DictionaryPlugin;
use crate::plugins::windows::HyprWindowsPlugin;
use crate::userinput::UserInput;

use super::pluginworker::PluginWorker;

pub enum DispatchMsg {
    UserInput(Arc<UserInput>)
}

pub struct PluginDispatcher {
    pub dispatcher_sender: flume::Sender<DispatchMsg>,
    dispatcher_receiver: flume::Receiver<DispatchMsg>,

    result_sender: flume::Sender<ResultMsg>,

    pool: futures::executor::ThreadPool,

    app_plugin: Arc<ApplicationPlugin>,
    window_plugin: Arc<HyprWindowsPlugin>,
    calc_plugin: Arc<CalculatorPlugin>,
    dict_plugin: Arc<DictionaryPlugin>,
    clip_plugin: Arc<ClipboardPlugin>
}

impl PluginDispatcher {
    pub fn new(directory_dir: &str,
               clipboard_path: &str,
               result_sender: &flume::Sender<ResultMsg>) {

        let window_sender = PluginWorker::launch(|| HyprWindowsPlugin::new(), result_sender.clone());
    }
}