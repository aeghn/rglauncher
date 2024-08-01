use std::{ops::Deref, sync::Arc, thread};

use flume::{Receiver, Sender};
use gpui::SharedString;
use rglcore::{
    misc::KTicket,
    plugins::{
        application::freedesktop::FDAppPlugin, calculator::CalculatorPlugin,
        hyprwindows::HyprWindowsPlugin, Plugin, PluginItem, PluginTrait,
    },
    userinput::UserInput,
};
use tracing::{error, info};

use crate::app::{self, RGLAppMsg};

#[derive(Clone)]
pub struct ArcPluginItem(Arc<PluginItem>);

impl Deref for ArcPluginItem {
    type Target = Arc<PluginItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<ArcPluginItem> for PluginItem {
    fn into(self) -> ArcPluginItem {
        ArcPluginItem(Arc::new(self))
    }
}

pub enum PluginDispatcherMsg {
    Filter(SharedString),
    PluginItems(KTicket, Vec<PluginItem>),
}

pub struct PluginDispatcher {
    ticket: KTicket,
    pub dp_tx: Sender<PluginDispatcherMsg>,
}

impl PluginDispatcher {
    pub fn new(app_tx: Sender<RGLAppMsg>) -> Self {
        let (dp_tx, dp_rx) = flume::unbounded::<PluginDispatcherMsg>();
        let ticket = KTicket::create();
        {
            let ticket: KTicket = ticket.clone();
            let dp_tx: Sender<PluginDispatcherMsg> = dp_tx.clone();
            thread::spawn(move || Self::run(app_tx, ticket, dp_tx, dp_rx));
        }

        Self { ticket, dp_tx }
    }

    async fn dispatch_loop(
        app_tx: Sender<RGLAppMsg>,
        ticket: KTicket,
        dp_tx: Sender<PluginDispatcherMsg>,
        dp_rx: Receiver<PluginDispatcherMsg>,
    ) {
        let mut plugins: Vec<Arc<Plugin>> = vec![];
        if let Ok(p) = FDAppPlugin::new() {
            plugins.push(Arc::new(p.into()));
        }
        if let Ok(p) = CalculatorPlugin::new() {
            plugins.push(Arc::new(p.into()));
        }
        if let Ok(p) = HyprWindowsPlugin::new() {
            plugins.push(Arc::new(p.into()));
        }

        let mut holder: Option<Arc<Vec<ArcPluginItem>>> = None;
        let mut ticket = ticket;

        while let Ok(msg) = dp_rx.recv_async().await {
            match msg {
                PluginDispatcherMsg::Filter(query) => {
                    holder.take();
                    ticket = ticket.create_next();
                    let input = Arc::new(UserInput::new(&query, &ticket));

                    for plugin in plugins.iter() {
                        let plugin = plugin.clone();
                        let input = input.clone();
                        let dp_tx = dp_tx.clone();
                        tokio::spawn(async move {
                            let result = plugin.handle_input(input.as_ref()).await;

                            if let Ok(rest) = result {
                                let _ = dp_tx
                                    .send_async(PluginDispatcherMsg::PluginItems(
                                        input.get_ticket(),
                                        rest,
                                    ))
                                    .await;
                            }
                        });
                    }
                }
                PluginDispatcherMsg::PluginItems(tic, items) => {
                    let mut old = if !tic.is_valid() {
                        continue;
                    } else if let Some(data) = &holder {
                        data.to_vec()
                    } else {
                        vec![]
                    };

                    old.extend(items.into_iter().map(|e| e.into()));
                    let new = Arc::new(old);

                    let _ = app_tx.send(RGLAppMsg::PluginItems(new.clone()));
                    holder.replace(new);
                }
            }
        }
    }

    fn run(
        app_tx: Sender<RGLAppMsg>,
        ticket: KTicket,
        dp_tx: Sender<PluginDispatcherMsg>,
        dp_rx: Receiver<PluginDispatcherMsg>,
    ) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(5)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move { Self::dispatch_loop(app_tx, ticket, dp_tx, dp_rx).await });
    }
}
