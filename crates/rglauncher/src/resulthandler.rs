use crate::sidebar::SidebarMsg;
use backend::plugindispatcher::DispatchMsg;
use backend::plugins::PluginResult;
use backend::userinput::UserInput;
use backend::ResultMsg;
use flume::Sender;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};
use crate::launcher::LauncherMsg;
use crate::pluginpreview::PreviewMsg;

pub struct ResultHolder {
    user_input: Option<Arc<UserInput>>,
    current_index: Option<u32>,
    result_holder: Vec<Arc<dyn PluginResult>>,
    result_id_set: HashSet<String>,

    pub result_sender: flume::Sender<ResultMsg>,
    result_receiver: flume::Receiver<ResultMsg>,

    launcher_sender: Sender<LauncherMsg>,
    dispatch_sender: flume::Sender<DispatchMsg>,

    sidebar_sender: flume::Sender<SidebarMsg>,
    preview_sender: Sender<PreviewMsg>,

    last: Instant,
}

impl ResultHolder {
    fn new(
        launcher_sender: Sender<LauncherMsg>,
        dispatch_sender: flume::Sender<DispatchMsg>,
        sidebar_sender: Sender<SidebarMsg>,
        preview_sender: Sender<PreviewMsg>,
    ) -> Self {
        let (result_sender, result_receiver) = flume::unbounded();

        Self {
            user_input: None,
            current_index: None,
            result_holder: vec![],
            result_id_set: HashSet::new(),

            result_sender,
            result_receiver,
            launcher_sender,
            dispatch_sender,
            sidebar_sender,
            preview_sender,
            last: Instant::now(),
        }
    }

    fn send_to_sidebar(&mut self) {
        self.result_holder
            .sort_by(|e1, e2| e2.score().cmp(&e1.score()));
        let holder = self.result_holder.clone();

        let holder_size = holder.len();
        self.sidebar_sender
            .send(SidebarMsg::Result(holder))
            .expect("unable to send result to sidebar");

        if holder_size == 0 {
            self.preview_sender.send(PreviewMsg::Clear)
                .expect("unable to clear preview");
        }

        self.last = Instant::now();
    }

    fn accept_messages(&mut self) {
        let interval = Duration::from_millis(80);
        let mut received_something = false;
        let mut next_sleep_time = 100000;
        loop {
            match self.result_receiver.recv_timeout(Duration::from_millis(next_sleep_time)) {
                Ok(msg) => match msg {
                    ResultMsg::Result(input, mut results) => match self.user_input.as_ref() {
                        None => {}
                        Some(user_input) => {
                            if user_input.as_ref() == input.as_ref() {
                                results.into_iter()
                                .for_each(|m| {
                                    if self.result_id_set.insert(m.get_id().to_string()) {
                                        self.result_holder.push(m);
                                    }
                                });
                                received_something = true;
                                next_sleep_time = 50;
                            }
                        }
                    },
                    ResultMsg::UserInput(input) => {
                        if let Some(mut old_input) = self.user_input.replace(input.clone()) {
                            old_input.cancel();
                            self.current_index.take();
                            self.result_holder.clear();
                            self.result_id_set.clear();
                        }
                        debug!("Send message to dispatcher: {}", input.input);
                        self.dispatch_sender
                            .send(DispatchMsg::UserInput(
                                input.clone(),
                                self.result_sender.clone(),
                            ))
                            .expect("todo");
                        self.last = Instant::now();
                    }
                    ResultMsg::RemoveWindow => {}
                    ResultMsg::ChangeSelect(item) => {
                        self.current_index.replace(item.clone());
                        match self.result_holder.get(item as usize) {
                            Some(pr) => {
                                self.preview_sender
                                    .send(PreviewMsg::PluginResult(pr.clone()))
                                    .expect("unable to send preview msg");
                            }
                            _ => {}
                        }
                    }
                    ResultMsg::SelectSomething => {
                        match self.current_index.clone() {
                            None => {}
                            Some(id) => {
                                match self.result_holder.get(id as usize) {
                                    Some(pr) => {
                                        pr.on_enter();
                                        self.launcher_sender.send(LauncherMsg::SelectSomething);
                                    }
                                    _ => {}
                                }
                            }
                        }

                    }
                },
                Err(ex) => {
                    // error!("unable to receive message: {:?}", ex);
                }
            }
            if received_something && Instant::now().duration_since(self.last).cmp(&interval).is_ge() {
                self.send_to_sidebar();
                received_something  = false;
                next_sleep_time = 100000;
            } else {
            }
        }
    }

    pub fn start(
        launcher_sender: &Sender<LauncherMsg>,
        dispatcher_sender: &flume::Sender<DispatchMsg>,
        sidebar_sender: &flume::Sender<SidebarMsg>,
        preview_sender: &Sender<PreviewMsg>,
    ) -> Sender<ResultMsg> {
        let mut result_handler = Self::new(
            launcher_sender.clone(),
            dispatcher_sender.clone(),
            sidebar_sender.clone(),
            preview_sender.clone(),
        );

        let result_sender = result_handler.result_sender.clone();

        thread::spawn(move || {
            result_handler.accept_messages();
        });

        result_sender
    }
}
