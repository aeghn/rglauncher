use flume::Receiver;
use glib::{BoxedAnyObject, StrV};

use crate::plugins::PluginResult;
use gtk::prelude::WidgetExt;
use gtk::PolicyType::Never;

#[derive(Clone)]
pub struct Preview {
    pub preview_window: gtk::ScrolledWindow,
}

impl Preview {
    pub fn new() -> Self {
        let preview_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(Never)
            .css_classes(StrV::from(["preview"]))
            .build();

        Preview { preview_window }
    }

    pub async fn loop_recv(&self, receiver: Receiver<BoxedAnyObject>) {
        loop {
            if let Ok(bao) = receiver.recv_async().await {
                let preview = bao.borrow::<Box<dyn PluginResult>>();
                let child = preview.preview();
                self.preview_window.set_vexpand(true);
                self.preview_window.set_hexpand(true);
                self.preview_window.set_child(Some(&child));
            }
        }
    }
}
