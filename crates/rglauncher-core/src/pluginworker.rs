use flume::Sender;
use futures::executor;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    plugins::{history::HistoryItem, Plugin, PluginMsg, PluginResult},
    ResultMsg,
};

pub struct PluginWorker<R, P, M>
where
    R: PluginResult,
    P: Plugin<R, M>,
    M: Send,
{
    pub message_tx: Sender<M>,
    _plugin: P,
    _phantom_data: std::marker::PhantomData<R>,
}

impl<R, P, M> PluginWorker<R, P, M>
where
    R: PluginResult + 'static,
    P: Plugin<R, M>,
    M: Send + Clone + 'static,
{
    pub fn launch<F>(name: &str, pluginbuilder: F) -> Sender<PluginMsg<M>>
    where
        F: Fn() -> P + 'static + Send,
    {
        let (message_tx, message_rx) = flume::unbounded::<PluginMsg<M>>();

        let name = name.to_owned();

        std::thread::Builder::new()
            .name(format!("rgworker-{}", name))
            .spawn(move || {
                let mut pool = executor::LocalPool::new();

                let mut plugin = pluginbuilder();
                info!("Finished creating plugin {}", name);

                pool.run_until(async {
                    loop {
                        if let Ok(plugin_msg) = message_rx.recv_async().await {
                            match plugin_msg {
                                PluginMsg::UserInput(user_input, result_tx, history) => {
                                    let mut trimmed = user_input.clone();
                                    if user_input.input.starts_with("/") {
                                        if let Some(index) = user_input.input.find(" ") {
                                            if !name.contains(&user_input.input[1..index]) {
                                                continue;
                                            } else {
                                                let mut trim_inner = user_input.as_ref().clone();
                                                trim_inner.input =
                                                    user_input.input.chars().skip(index).collect();
                                                trimmed = Arc::new(trim_inner);
                                            }
                                        } else if name.contains(&user_input.input[1..]) {
                                            let mut trim_inner = user_input.as_ref().clone();
                                            trim_inner.input = "".to_string();
                                            trimmed = Arc::new(trim_inner);
                                        }
                                    }

                                    let this_types: Vec<&HistoryItem> = history
                                        .iter()
                                        .filter(|h| h.plugin_type == plugin.get_type_id())
                                        .collect();

                                    let typed = if this_types.len() > 0 {
                                        Some(this_types)
                                    } else {
                                        None
                                    };

                                    match plugin.handle_input(trimmed.as_ref(), typed) {
                                        Ok(vec) => {
                                            let upcast: Vec<Arc<dyn PluginResult>> = vec
                                                .into_iter()
                                                .map(|e| Arc::new(e) as Arc<dyn PluginResult>)
                                                .collect();
                                            if let Err(err) = result_tx
                                                .send(ResultMsg::Result(user_input.clone(), upcast))
                                            {
                                                error!("{}", err);
                                            }
                                        }
                                        Err(_) => {}
                                    };
                                }
                                PluginMsg::RefreshContent => {
                                    plugin.refresh_content();
                                }
                                PluginMsg::TypeMsg(type_msg) => {
                                    plugin.handle_msg(type_msg);
                                }
                            }
                        }
                    }
                })
            })
            .unwrap();

        message_tx
    }
}
