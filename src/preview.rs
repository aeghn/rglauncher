use flume::Receiver;
use glib::{BoxedAnyObject, Cast, clone, ControlFlow, StrV};
use std::collections::HashMap;

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
        let preview_window = self.preview_window.clone();
        loop {
            if let Ok(bao) = receiver.recv_async().await {
                glib::idle_add_local(clone!(@strong preview_window => move || {
                    let down = bao.try_borrow::<Box<dyn PluginResult>>();
                    if let Ok(pr) = down {
                        let child = pr.preview();
                        preview_window.set_child(Some(&child));
                    } else {
                        preview_window.set_child(None::<&gtk::Widget>);
                    }
                    ControlFlow::Break
                }));

            }
        }
    }
}
