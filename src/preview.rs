use std::cell::Ref;
use flume::Receiver;
use glib::{BoxedAnyObject, Cast};
use gtk::PolicyType::Never;
use crate::plugins::PluginResult;

#[derive(Clone)]
pub struct Preview {
    pub preview_window: gtk::ScrolledWindow,
    selection_change_receiver: Receiver<BoxedAnyObject>
}

impl Preview {
    pub fn new(selection_change_receiver: Receiver<BoxedAnyObject>) -> Self {
        let preview_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(Never)
            .build();

        Preview {
            preview_window,
            selection_change_receiver,
        }
    }

    pub async fn loop_recv(&self) {
        let receiver = self.selection_change_receiver.clone();
        // loop {
            // if let Ok(bao) = receiver.recv_async().await {
            //     let preview = bao.borrow::<Box<dyn PluginResult>>();
            //     self.preview_window.set_child(Some(&preview.preview()));
            // }
        // }
    }
}