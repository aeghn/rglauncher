use crate::arguments::Arguments;
use crate::pluginpreview::app::AppPreview;
use crate::pluginpreview::calculator::CalcPreview;
use crate::pluginpreview::clipboard::ClipPreview;
use crate::pluginpreview::dictionary::DictPreview;
use crate::pluginpreview::windows::HyprWindowPreview;
use anyhow::anyhow;
use backend::plugins::app::AppResult;
use backend::plugins::calculator::CalcResult;
use backend::plugins::clipboard::ClipResult;
use backend::plugins::dict::{self, DictResult};
use backend::plugins::windows::HyprWindowResult;
use backend::plugins::PluginResult;
use glib::{clone, MainContext};
use gtk::pango::WrapMode::{Word, WordChar};
use gtk::prelude::{GridExt, WidgetExt};
use gtk::Align::Center;
use gtk::ResponseType::No;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info;

mod app;
mod calculator;
mod clipboard;
mod dictionary;
mod windows;

const DEFAULT_ID: &str = "default";

pub trait PluginPreview {
    type PluginResult: PluginResult;

    fn new() -> Self
    where
        Self: Sized;

    fn get_preview(&self) -> gtk::Widget;

    fn set_preview(&self, plugin_result: &Self::PluginResult);

    fn get_id(&self) -> &str;
}

pub struct PluginPreviewBuilder {
    stack: gtk::Stack,

    app_preview: AppPreview,
    calc_preview: CalcPreview,
    clip_preview: ClipPreview,
    dict_preview: DictPreview,
    wind_preview: HyprWindowPreview,
}

impl PluginPreviewBuilder {
    pub fn new(stack: &gtk::Stack, arguments: Arc<Arguments>) -> Self {
        let dict_preview = DictPreview::new();
        let app_preview = AppPreview::new();
        let calc_preview = CalcPreview::new();
        let clip_preview = ClipPreview::new();
        let wind_preview = HyprWindowPreview::new();

        stack.add_named(&dict_preview.get_preview(), Some(dict_preview.get_id()));
        stack.add_named(&app_preview.get_preview(), Some(app_preview.get_id()));
        stack.add_named(&calc_preview.get_preview(), Some(calc_preview.get_id()));
        stack.add_named(&clip_preview.get_preview(), Some(clip_preview.get_id()));
        stack.add_named(&wind_preview.get_preview(), Some(wind_preview.get_id()));

        let default = gtk::Label::builder()
            .label(glib::GString::from("RGLauncher"))
            .vexpand(true)
            .hexpand(true)
            .valign(Center)
            .halign(Center)
            .css_classes(["logo-font", "dim-label"])
            .build();
        stack.add_named(&default, Some(DEFAULT_ID));
        stack.set_visible_child(&default);

        dict_preview.add_csses(arguments.dict_dir.as_str());

        PluginPreviewBuilder {
            stack: stack.clone(),
            app_preview,
            calc_preview,
            clip_preview,
            dict_preview,
            wind_preview,
        }
    }

    pub fn set_preview(&self, opr: Option<&Arc<dyn PluginResult>>) -> Option<()> {
        if let Some(plugin_result) = opr {
            let any = plugin_result.as_any();

            let preview_id = plugin_result.get_type_id();
            match preview_id {
                backend::plugins::windows::TYPE_ID => {
                    let win = any.downcast_ref::<HyprWindowResult>()?;
                    self.wind_preview.set_preview(win);
                }

                backend::plugins::app::TYPE_ID => {
                    let win = any.downcast_ref::<AppResult>()?;
                    self.app_preview.set_preview(win);
                }

                backend::plugins::calculator::TYPE_ID => {
                    let win = any.downcast_ref::<CalcResult>()?;
                    self.calc_preview.set_preview(win);
                }

                backend::plugins::clipboard::TYPE_ID => {
                    let win = any.downcast_ref::<ClipResult>()?;
                    self.clip_preview.set_preview(win);
                }

                backend::plugins::dict::TYPE_ID => {
                    let win = any.downcast_ref::<DictResult>()?;
                    self.dict_preview.set_preview(win);
                }

                _ => {}
            };

            self.stack.set_visible_child_name(preview_id);
        } else {
            self.stack.set_visible_child_name(DEFAULT_ID);
        }

        None
    }
}

fn build_pair_line(grid: &gtk::Grid, row: i32, title: &str) -> gtk::Label {
    let left = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .wrap(true)
        .wrap_mode(WordChar)
        .label(title)
        .hexpand(true)
        .build();
    left.add_css_class("dim-label");
    left.add_css_class("info-label");
    let right = gtk::Label::builder()
        .halign(gtk::Align::End)
        .wrap(true)
        .wrap_mode(WordChar)
        .build();
    grid.attach(&left, 0, row, 1, 1);
    grid.attach(&right, 1, row, 1, 1);

    right
}

fn get_seprator() -> gtk::Separator {
    gtk::Separator::builder()
        .hexpand(true)
        .css_classes(["horizontal-separator"])
        .build()
}

pub enum PreviewMsg {
    PluginResult(Arc<dyn PluginResult>),
    Clear,
}

#[derive(Clone)]
pub struct Preview {
    pub preview_sender: flume::Sender<PreviewMsg>,
    preview_receiver: flume::Receiver<PreviewMsg>,

    pub preview_window: gtk::Stack,
}

impl Preview {
    pub fn new(
        preview_sender: flume::Sender<PreviewMsg>,
        preview_receiver: flume::Receiver<PreviewMsg>,
    ) -> Self {
        let preview_window = gtk::Stack::builder()
            .vexpand(true)
            .hexpand(true)
            .css_classes(["preview"])
            .build();

        Preview {
            preview_sender,
            preview_receiver,
            preview_window,
        }
    }

    pub fn loop_recv(&self, arguments: &Arc<Arguments>) {
        let preview_window = self.preview_window.clone();
        let preview_receiver = self.preview_receiver.clone();
        let arguments = arguments.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            let plugin_previews = Rc::new(RefCell::new(PluginPreviewBuilder::new(
                &preview_window,
                arguments,
            )));
            loop {
                if let Ok(preview_msg) = preview_receiver.recv_async().await {
                    let plugin_preview_builder = plugin_previews.clone();
                    let opt_plugin_result = match preview_msg {
                        PreviewMsg::PluginResult(pr) => Some(pr),
                        PreviewMsg::Clear => None,
                    };
                    glib::idle_add_local_once(clone!(@strong preview_window => move || {
                        plugin_preview_builder.borrow().set_preview(opt_plugin_result.as_ref());
                    }));
                }
            }
        });
    }
}
