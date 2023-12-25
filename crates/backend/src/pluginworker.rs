use futures::executor;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    plugins::{Plugin, PluginMsg, PluginResult},
    ResultMsg,
};

pub struct PluginWorker<R, P, M>
where
    R: PluginResult,
    P: Plugin<R, M>,
    M: Send,
{
    pool: executor::LocalPool,
    plugin: P,
    pub message_sender: flume::Sender<M>,
    message_receiver: flume::Receiver<M>,
    _phantom_data: std::marker::PhantomData<R>,
}

impl<R, P, M> PluginWorker<R, P, M>
where
    R: PluginResult + 'static,
    P: Plugin<R, M>,
    M: Send + 'static,
{
    pub fn launch<F>(pluginbuilder: F) -> flume::Sender<PluginMsg<M>>
    where
        F: Fn() -> P + 'static + Send,
    {
        let (message_sender, message_receiver) = flume::unbounded::<PluginMsg<M>>();
        {
            let message_receiver = message_receiver.clone();
            std::thread::spawn(move || {
                let mut pool = executor::LocalPool::new();

                let mut plugin = pluginbuilder();
                info!("Finished creating plugin");

                pool.run_until(async {
                    loop {
                        if let Ok(plugin_msg) = message_receiver.recv_async().await {
                            match plugin_msg {
                                PluginMsg::UserInput(user_input, result_sender) => {
                                    match plugin.handle_input(user_input.as_ref()) {
                                        Ok(vec) => {
                                            let upcast: Vec<Arc<dyn PluginResult>> = vec
                                                .into_iter()
                                                .map(|e| Arc::new(e) as Arc<dyn PluginResult>)
                                                .collect();
                                            // info!(
                                            //     "Send Result: {} {}",
                                            //     plugin.get_type_id(),
                                            //     &upcast.len()
                                            // );
                                            if let Err(err) = result_sender
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
            });
        }

        message_sender
    }
}
