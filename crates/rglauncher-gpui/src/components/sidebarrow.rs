use gpui::*;
use rglcore::plugins::{PluginItem, PluginItemTrait};

use crate::{plugindispatcher::ArcPluginItem, state::StateModel};

impl IntoElement for ArcPluginItem {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let base = div()
            .flex()
            .justify_between()
            .items_center()
            .py_2()
            .px_4()
            .border_t_1()
            .text_xl();

        match self.as_ref() {
            PluginItem::MDict(res) => base.child(res.word.clone()),
            PluginItem::HyprWin(res) => base.child(res.title.clone()),
            PluginItem::Clip(res) => base.child(res.content.clone()),
            PluginItem::Calc(res) => base.child(res.formula.clone()),
            PluginItem::App(res) => base.child(res.app_name.to_string()),
        }
    }
}

impl RenderOnce for ArcPluginItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        self.into_element()
    }
}
