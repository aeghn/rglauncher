use std::sync::Arc;

use arc_swap::{access::Access, ArcSwap};
use gpui::*;
use tracing::info;

use crate::plugindispatcher::ArcPluginItem;

#[derive(Clone)]
pub struct State {
    pub items: Arc<ArcSwap<Vec<ArcPluginItem>>>,
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Model<State>,
}

impl StateModel {
    pub fn init(cx: &mut WindowContext) {
        let model = cx.new_model(|_cx| State {
            items: Default::default(),
        });
        let this = Self { inner: model };
        cx.set_global(this.clone());
    }

    pub fn update(f: impl FnOnce(&mut Self, &mut WindowContext), cx: &mut WindowContext) {
        if !cx.has_global::<Self>() {
            tracing::error!("StateModel not found");
            return;
        }
        cx.update_global::<Self, _>(|this, cx| {
            f(this, cx);
        });
    }

    pub fn update_async(
        f: impl FnOnce(&mut Self, &mut WindowContext),
        cx: &mut AsyncWindowContext,
    ) {
        let _ = cx.update_global::<Self, _>(|this, cx| {
            f(this, cx);
        });
    }

    pub fn swap_items(&self, items: Arc<Vec<ArcPluginItem>>, cx: &mut WindowContext) {
        self.inner.update(cx, |model, cx| {
            model.items.swap(items);

            cx.emit(ListChangedEvent {});
        });
    }
}

impl Global for StateModel {}

#[derive(Clone, Debug)]
pub struct ListChangedEvent {}

impl EventEmitter<ListChangedEvent> for State {}
