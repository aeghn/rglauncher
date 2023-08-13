use std::collections::HashMap;
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
            .vexpand(true)
            .hexpand(true)
            .build();

        Preview { preview_window }
    }

    pub async fn loop_recv(&self, receiver: Receiver<BoxedAnyObject>) {
        let mut view_map: HashMap<&str, gtk::Widget> = HashMap::new();

        loop {
            if let Ok(bao) = receiver.recv_async().await {
                let preview = bao.borrow::<Box<dyn PluginResult>>();
                let child = preview.preview();
                self.preview_window.set_child(Some(&child));
            }
        }
    }
}
