use crate::pluginpreview::app::AppPreview;
use crate::pluginpreview::calculator::CalcPreview;
use crate::pluginpreview::clipboard::ClipPreview;
use crate::pluginpreview::dictionary::DictPreview;
use crate::pluginpreview::windows::HyprWindowPreview;
use anyhow::anyhow;
use backend::plugins::app::AppResult;
use backend::plugins::calculator::CalcResult;
use backend::plugins::clipboard::ClipResult;
use backend::plugins::dict::DictResult;
use backend::plugins::windows::HyprWindowResult;
use backend::plugins::PluginResult;
use gtk::ResponseType::No;
use std::any::Any;
use std::sync::Arc;
use tracing::info;

mod app;
mod calculator;
mod clipboard;
mod dictionary;
mod windows;

pub trait PluginPreview {
    type PluginResult: PluginResult;

    fn new() -> Self
    where
        Self: Sized;

    fn get_preview(&self, plugin_result: &Self::PluginResult) -> gtk::Widget;
}

pub struct PluginPreviewBuilder {
    app_preview: AppPreview,
    calc_preview: CalcPreview,
    clip_preview: ClipPreview,
    dict_preview: DictPreview,
    wind_preview: HyprWindowPreview,
}

impl PluginPreviewBuilder {
    pub fn new() -> Self {
        PluginPreviewBuilder {
            app_preview: AppPreview::new(),
            calc_preview: CalcPreview::new(),
            clip_preview: ClipPreview::new(),
            dict_preview: DictPreview::new(),
            wind_preview: HyprWindowPreview::new(),
        }
    }

    pub fn get_preview(&self, plugin_result: &Arc<dyn PluginResult>) -> Option<gtk::Widget> {
        let any = plugin_result.as_any();

        match plugin_result.get_type_id() {
            backend::plugins::windows::TYPE_ID => {
                let win = any.downcast_ref::<HyprWindowResult>()?;
                Some(self.wind_preview.get_preview(win))
            }

            backend::plugins::app::TYPE_ID => {
                let win = any.downcast_ref::<AppResult>()?;
                Some(self.app_preview.get_preview(win))
            }

            backend::plugins::calculator::TYPE_ID => {
                let win = any.downcast_ref::<CalcResult>()?;
                Some(self.calc_preview.get_preview(win))
            }

            backend::plugins::clipboard::TYPE_ID => {
                let win = any.downcast_ref::<ClipResult>()?;
                Some(self.clip_preview.get_preview(win))
            }

            backend::plugins::dict::TYPE_ID => {
                let win = any.downcast_ref::<DictResult>()?;
                Some(self.dict_preview.get_preview(win))
            }

            _ => None,
        }
    }
}
