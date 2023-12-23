use glib::{clone, BoxedAnyObject, ControlFlow, MainContext, StrV};
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::pluginpreview::PluginPreviewBuilder;
use backend::plugins::PluginResult;
use gtk::prelude::WidgetExt;
use gtk::PolicyType::Never;

pub enum PreviewMsg {
    PluginResult(Arc<dyn PluginResult>),
}

#[derive(Clone)]
pub struct Preview {
    pub preview_sender: flume::Sender<PreviewMsg>,
    preview_receiver: flume::Receiver<PreviewMsg>,

    pub preview_window: gtk::ScrolledWindow,
}

impl Preview {
    pub fn new(
        preview_sender: flume::Sender<PreviewMsg>,
        preview_receiver: flume::Receiver<PreviewMsg>,
    ) -> Self {
        let preview_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(Never)
            .css_classes(StrV::from(["preview"]))
            .vexpand(true)
            .hexpand(true)
            .build();

        Preview {
            preview_sender,
            preview_receiver,
            preview_window,
        }
    }

    pub fn loop_recv(&self) {
        let preview_window = self.preview_window.clone();
        let preview_receiver = self.preview_receiver.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            let plugin_preview_builder = Rc::new(RefCell::new(PluginPreviewBuilder::new()));
            loop {
                if let Ok(preview_msg) = preview_receiver.recv_async().await {
                    match preview_msg {
                        PreviewMsg::PluginResult(pr) => {
                            let plugin_preview_builder = plugin_preview_builder.clone();
                            glib::idle_add_local_once(clone!(@strong preview_window => move || {
                            let preview = plugin_preview_builder.borrow().get_preview(&pr);

                                preview_window.set_child(preview.as_ref());

                            }));
                        }
                    }
                }
            }
        });
    }
}
