use flume::{Receiver, Sender};
use futures::executor;
use std::{
    any::Any,
    sync::{Arc, RwLock},
};
use tracing::{error, info};

use crate::{
    plugins::{history::HistoryItem, Plugin, PluginResult},
    userinput::UserInput,
    util::score_utils,
    ResultMsg,
};

use super::{DispatchMsg, InnerDispatcher};

pub struct PluginWorker<R, P, M, F>
where
    R: PluginResult,
    P: Plugin<R, M>,
    M: Send,
    F: Fn() -> anyhow::Result<P> + 'static + Send,
{
    pub plugin: P,
    _phantom_m: std::marker::PhantomData<M>,
    _phantom_r: std::marker::PhantomData<R>,
    _phantom_f: std::marker::PhantomData<F>,
}

impl<R, P, M, F> PluginWorker<R, P, M, F>
where
    R: PluginResult + 'static,
    P: Plugin<R, M>,
    M: Send + Clone + 'static,
    F: Fn() -> anyhow::Result<P> + 'static + Send,
{
    pub fn launch(pluginbuilder: F, inner_dispatcher: &InnerDispatcher, plugin_type: &'static str) {
        let inner: InnerDispatcher = inner_dispatcher.clone();
        std::thread::Builder::new()
            .name(format!("rgworker-{}", plugin_type))
            .spawn(move || Self::new_and_work(pluginbuilder, inner, plugin_type))
            .unwrap();
    }

    fn new_and_work(pluginbuilder: F, inner: InnerDispatcher, plugin_type: &str) {
        let plugin = pluginbuilder();
        let mut inner = inner;
        match plugin {
            Ok(plugin) => {
                let mut worker = PluginWorker {
                    plugin,
                    _phantom_m: std::marker::PhantomData,
                    _phantom_r: std::marker::PhantomData,
                    _phantom_f: std::marker::PhantomData::<F>,
                };

                let mut pool = executor::LocalPool::new();
                pool.run_until(async {
                    loop {
                        if let Ok(plugin_msg) = inner.dispatch_rx.recv().await {
                            match plugin_msg {
                                DispatchMsg::UserInput(ui, tx) => {
                                    worker.handle_user_input(&ui, tx, &inner.history)
                                }
                                DispatchMsg::RefreshContent => worker.handle_refresh_content(),
                                DispatchMsg::PluginMsg => worker.handle_plugin_msg(),
                                _ => {}
                            }
                        }
                    }
                })
            }
            Err(err) => {
                error!("unable to create plugin {}, {}", plugin_type, err);
            }
        }
    }

    fn handle_user_input(
        &self,
        user_input: &Arc<UserInput>,
        result_tx: flume::Sender<ResultMsg>,
        history: &Arc<RwLock<Vec<HistoryItem>>>,
    ) {
        let type_id = self.plugin.get_type_id();

        let mut trimmed = user_input.clone();
        if user_input.input.starts_with("/") {
            if let Some(index) = user_input.input.find(" ") {
                if !type_id.contains(&user_input.input[1..index]) {
                    return;
                } else {
                    let mut trim_inner = user_input.as_ref().clone();
                    trim_inner.input = user_input.input.chars().skip(index).collect();
                    trimmed = Arc::new(trim_inner);
                }
            } else if type_id.contains(&user_input.input[1..]) {
                let mut trim_inner = user_input.as_ref().clone();
                trim_inner.input = "".to_string();
                trimmed = Arc::new(trim_inner);
            }
        }

        let this_types: Option<Vec<HistoryItem>> = match history.read() {
            Ok(vec) => Some(
                vec.iter()
                    .enumerate()
                    .filter(|h| h.1.plugin_type == self.plugin.get_type_id())
                    .map(|e| HistoryItem {
                        score: score_utils::highest((10000 - e.0 % 10000).try_into().unwrap()),
                        ..e.1.clone()
                    })
                    .collect(),
            ),
            Err(_) => None,
        };

        if let Ok(vec) = self.plugin.handle_input(trimmed.as_ref(), this_types) {
            let upcast: Vec<Arc<dyn PluginResult>> = vec
                .into_iter()
                .map(|e| Arc::new(e) as Arc<dyn PluginResult>)
                .collect();
            if let Err(err) = result_tx.send(ResultMsg::Result(user_input.clone(), upcast)) {
                error!("{}", err);
            }
        };
    }

    fn handle_refresh_content(&mut self) {
        self.plugin.refresh_content();
    }

    fn handle_plugin_msg(&mut self) {}
}
