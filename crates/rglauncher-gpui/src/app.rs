use std::sync::Arc;

use crate::container::sidebar::{SelectDown, SelectUp, Sidebar};
use crate::plugindispatcher::PluginDispatcherMsg;
use crate::theme::*;
use crate::{container::inputcontrol::InputControl, plugindispatcher::ArcPluginItem};
use flume::{Receiver, Sender};
use gpui::*;

pub enum RGLAppMsg {
    FilterChanged(SharedString),
    PluginItems(Arc<Vec<ArcPluginItem>>),
}

pub struct RGLApp {
    pub list_view: View<Sidebar>,
    pub input_view: View<InputControl>,
    receiver: Receiver<RGLAppMsg>,
}

impl RGLApp {
    pub fn new(
        cx: &mut WindowContext,
        pd_tx: Sender<PluginDispatcherMsg>,
        app_rx: Receiver<RGLAppMsg>,
    ) -> View<Self> {
        let list_view = Sidebar::new(cx);
        let input_view = InputControl::new(cx, &pd_tx);
        let view = cx.new_view(move |_| RGLApp {
            list_view,
            input_view,
            receiver: app_rx,
        });

        view
    }
}

impl Render for RGLApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let list = div()
            .flex()
            .flex_grow()
            .justify_center()
            .items_center()
            .child(self.list_view.clone());

        let controls = div()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(theme.crust_light)
            .child(
                div()
                    .flex()
                    .gap_1()
                    .mb_2()
                    .mx_2()
                    .child(self.input_view.clone()),
            );

        let app = div()
            .flex()
            .flex_grow()
            .flex_col()
            .size_full()
            .justify_between()
            .gap_1()
            .child(controls)
            .child(list);

        div()
            .rounded_xl()
            .border_1()
            .border_color(theme.overlay0)
            .size_full()
            .child(
                div()
                    .bg(theme.base_blur)
                    .rounded_xl()
                    .flex()
                    .flex_col()
                    .size_full()
                    .justify_between()
                    .text_color(theme.text)
                    .child(app),
            )
    }
}
