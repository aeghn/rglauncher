use crate::launcher::LauncherMsg;
use crate::pluginpreview::PreviewMsg;
use crate::sidebar::SidebarMsg;
use chin_tools::SharedStr;
use flume::{Receiver, Sender};
use rglcore::dispatcher::DispatchMsg;
use rglcore::plugins::{PRWrapper, PluginResult};
use rglcore::userinput::UserInput;
use rglcore::ResultMsg;
use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error};

pub struct ResultHolder {
    user_input: Option<UserInput>,
    current_index: Option<u32>,
    result_holder: Vec<PRWrapper>,
    result_id_set: HashSet<SharedStr>,

    pub result_tx: Sender<ResultMsg>,
    result_rx: Receiver<ResultMsg>,

    launcher_tx: Sender<LauncherMsg>,
    dispatch_tx: flume::Sender<DispatchMsg>,

    sidebar_tx: Sender<SidebarMsg>,
    preview_tx: Sender<PreviewMsg>,

    last: Instant,
}

impl ResultHolder {
    fn new(
        launcher_tx: &Sender<LauncherMsg>,
        dispatch_tx: &flume::Sender<DispatchMsg>,
        sidebar_tx: &Sender<SidebarMsg>,
        preview_tx: &Sender<PreviewMsg>,
    ) -> Self {
        let (result_tx, result_rx) = flume::unbounded();

        Self {
            user_input: None,
            current_index: None,
            result_holder: vec![],
            result_id_set: HashSet::new(),

            result_tx,
            result_rx,
            launcher_tx: launcher_tx.clone(),
            dispatch_tx: dispatch_tx.clone(),
            sidebar_tx: sidebar_tx.clone(),
            preview_tx: preview_tx.clone(),
            last: Instant::now(),
        }
    }

    fn send_to_sidebar(&mut self) {
        self.result_holder.sort_by(|e1, e2| e2.score.cmp(&e1.score));
        let holder = self.result_holder.clone();

        let holder_size = holder.len();
        self.sidebar_tx
            .send(SidebarMsg::Result(holder))
            .expect("unable to send result to sidebar");

        if holder_size == 0 {
            self.preview_tx
                .send(PreviewMsg::Clear)
                .expect("unable to clear preview");
        }

        self.last = Instant::now();
    }

    fn accept_messages(&mut self) {
        let interval = Duration::from_millis(30);
        let mut received_something = false;
        let mut next_sleep_time = 100000;
        loop {
            match self
                .result_rx
                .recv_timeout(Duration::from_millis(next_sleep_time))
            {
                Ok(msg) => match msg {
                    ResultMsg::Result(input, results) => match self.user_input.as_ref() {
                        None => {}
                        Some(user_input) => {
                            if user_input == input.as_ref() {
                                results.into_iter().for_each(|m| {
                                    if self.result_id_set.insert(m.get_id().into()) {
                                        self.result_holder.push(m);
                                    }
                                });
                                received_something = true;
                                next_sleep_time = 50;
                            }
                        }
                    },
                    ResultMsg::UserInput(input) => {
                        if let Some(old_input) = self.user_input.replace(input.clone()) {
                            if !old_input.cancelled() {
                                old_input.cancel();
                            }
                            self.current_index.take();
                            self.result_holder.clear();
                            self.result_id_set.clear();
                        }
                        debug!("Send message to dispatcher: {:?}", input.input);
                        match self
                            .dispatch_tx
                            .send(DispatchMsg::UserInput(input.into(), self.result_tx.clone()))
                        {
                            Ok(_) => {
                                self.last = Instant::now();
                            }
                            Err(err) => {
                                error!("unable send to dispatcher {}", err);
                                break;
                            }
                        }
                    }

                    ResultMsg::RemoveWindow => {}
                    ResultMsg::ChangeSelect(item) => {
                        self.current_index.replace(item.clone());
                        match self.result_holder.get(item as usize) {
                            Some(pr) => {
                                self.preview_tx
                                    .send(PreviewMsg::PluginResult(pr.clone()))
                                    .expect("unable to send preview msg");
                            }
                            _ => {}
                        }
                    }
                    ResultMsg::SelectSomething => match self.current_index.clone() {
                        None => {}
                        Some(id) => match self.result_holder.get(id as usize) {
                            Some(pr) => {
                                pr.on_enter();
                                self.launcher_tx
                                    .send(LauncherMsg::SelectSomething)
                                    .expect("unable to send select");
                                self.dispatch_tx
                                    .send(DispatchMsg::SetHistory(pr.clone()))
                                    .expect("unable to set history");
                            }
                            _ => {}
                        },
                    },
                },
                Err(_ex) => {
                    // error!("unable to receive message: {:?}", ex);
                }
            }
            if received_something
                && Instant::now()
                    .duration_since(self.last)
                    .cmp(&interval)
                    .is_ge()
            {
                self.send_to_sidebar();
                received_something = false;
                next_sleep_time = 100000;
            }
        }
    }

    pub fn start(
        launcher_tx: &Sender<LauncherMsg>,
        dispatch_tx: &flume::Sender<DispatchMsg>,
        sidebar_tx: &Sender<SidebarMsg>,
        preview_tx: &Sender<PreviewMsg>,
    ) -> Sender<ResultMsg> {
        let mut result_handler = Self::new(launcher_tx, dispatch_tx, sidebar_tx, preview_tx);

        let result_tx = result_handler.result_tx.clone();

        thread::spawn(move || {
            result_handler.accept_messages();
        });

        result_tx
    }
}
