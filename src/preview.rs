use std::cell::Ref;
use flume::Receiver;
use glib::{BoxedAnyObject, Cast, StrV};
use gtk::Align::Center;
use gtk::PolicyType::Never;
use gtk::prelude::WidgetExt;
use crate::plugins::PluginResult;

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

        Preview {
            preview_window,
        }
    }

    pub async fn loop_recv(&self, receiver: Receiver<BoxedAnyObject>) {
        loop {
            if let Ok(bao) = receiver.recv_async().await {
                let preview = bao.borrow::<Box<dyn PluginResult>>();
                let child = preview.preview();
                child.set_hexpand(true);
                child.set_vexpand(true);
                child.set_valign(Center);
                child.set_halign(Center);
                self.preview_window.set_child(Some(&child));
            }
        }
    }
}