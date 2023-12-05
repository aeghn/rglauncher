use std::collections::HashMap;
use flume::Receiver;
use glib::{BoxedAnyObject, clone, ControlFlow, StrV};


use crate::plugins::{PluginPreview, PluginResult, self};
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
            if let Ok(gboxed) = receiver.recv_async().await {
                glib::idle_add_local(clone!(@strong preview_window => move || {
                    let down = gboxed.try_borrow::<Box<dyn PluginResult>>();
                    if let Ok(plugin_result) = down {
                  /*       let child = plugin_result.preview(); */
           /*              preview_window.set_child(Some(&child)); */
                    } else {
                        preview_window.set_child(None::<&gtk::Widget>);
                    }
                    ControlFlow::Break
                }));

            }
        }
    }
}
