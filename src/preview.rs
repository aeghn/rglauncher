use flume::Receiver;
use glib::{clone, BoxedAnyObject, ControlFlow, StrV};
use std::any::TypeId;
use std::collections::HashMap;

use crate::plugins::app::{AppResult, AppPreview, self};
use crate::plugins::calculator::{CalcPreview, CalcResult};
use crate::plugins::{self, PluginPreview, PluginResult};
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
        let app_preview = app::AppPreview::new();
        loop {
            if let Ok(gboxed) = receiver.recv_async().await {
                glib::idle_add_local_once(clone!(@strong preview_window => move || {
                    let down = gboxed.try_borrow::<Box<dyn PluginResult>>();
                    if let Ok(plugin_result) = down {
                        match plugin_result.get_type_id() {
                            _ => {
                                preview_window.set_child(None::<&gtk::Widget>);
                            }
                        }
                    } else {
                        preview_window.set_child(None::<&gtk::Widget>);
                    }
                }));
            }
        }
    }
}
