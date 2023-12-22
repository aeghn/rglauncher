use std::os;
use std::sync::Arc;
use crate::ResultMsg;
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


}

impl PluginDispatcher {
    pub fn new(dictionary_dir: &str,
               clipboard_path: &str,
               result_sender: &flume::Sender<ResultMsg>) {

        let dictionary_dir = dictionary_dir.to_string();
        let clipboard_path= clipboard_path.to_string();

        let window_sender = PluginWorker::launch(|| HyprWindowsPlugin::new(), result_sender.clone());
        let app_sender = PluginWorker::launch(|| ApplicationPlugin::new(), result_sender.clone());
        let clip_sender = PluginWorker::launch(move || ClipboardPlugin::new(clipboard_path.as_str()), result_sender.clone());
        let dict_sender = PluginWorker::launch(move || DictionaryPlugin::new(dictionary_dir.as_str()), result_sender.clone());
        let calc_sender = PluginWorker::launch(|| CalculatorPlugin::new(), result_sender.clone());


    }
}