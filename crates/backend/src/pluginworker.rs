use futures::executor;

use crate::plugins::{PluginMsg, ResultMsg, Plugin, PluginResult};

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
    R: PluginResult,
    P: Plugin<R, M>,
    M: Send + 'static,
{
    pub fn launch<F>(
        pluginbuilder: F,
        result_sender: flume::Sender<ResultMsg>,
    ) -> flume::Sender<PluginMsg<M>>
    where
        F: Fn() -> P + 'static + Send,
    {
        let (message_sender, message_receiver) = flume::unbounded::<PluginMsg<M>>();
        {
            let message_receiver = message_receiver.clone();
            std::thread::spawn(move || {
                let mut pool = executor::LocalPool::new();
                let mut plugin = pluginbuilder();

                pool.run_until(async {
                    loop {
                        if let Ok(plugin_msg) = message_receiver.recv_async().await {
                            match plugin_msg {
                                PluginMsg::UserInput(user_input) => {
                                    let result = plugin.handle_input(user_input.as_ref());
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
