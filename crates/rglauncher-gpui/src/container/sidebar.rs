use crate::state::StateModel;
use gpui::*;
use tracing::error;

actions!(sidebar, [SelectUp, SelectDown]);

pub struct Sidebar {
    state: ListState,
    cursor: Model<usize>,
}

impl Render for Sidebar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::up))
            .child(list(self.state.clone()).w_full().h_full())
    }
}

impl Sidebar {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let state = cx.global::<StateModel>().inner.clone();
            cx.subscribe(&state, |this: &mut Sidebar, model, _event, cx| {
                let items = model.read(cx).items.load();
                let cursor = *this.cursor.read(cx);
                this.state = ListState::new(
                    items.len(),
                    ListAlignment::Top,
                    Pixels(20.),
                    move |idx, _cx| {
                        let item = items.get(idx).unwrap().clone();
                        if cursor == idx {
                            div()
                                .child(item)
                                .bg(Fill::Color(Hsla::red()))
                                .into_any_element()
                        } else {
                            div().child(item).into_any_element()
                        }
                    },
                );
                cx.notify();
            })
            .detach();

            let cursor = cx.new_model(|_cx| 0);

            Sidebar {
                state: ListState::new(0, ListAlignment::Bottom, Pixels(20.), move |_, _| {
                    div().into_any_element()
                }),
                cursor,
            }
        })
    }

    pub fn up(&mut self, _: &SelectUp, cx: &mut ViewContext<Self>) {
        let index = *self.cursor.read(cx);

        let new = index.saturating_sub(1);

        error!("up {}", new);

        self.state.scroll_to_reveal_item(index);
        self.cursor.update(cx, |e, v| {
            *e = new;
        });
        cx.notify()
    }

    pub fn down(&mut self, _: &SelectDown, cx: &mut ViewContext<Self>) {
        let index = *self.cursor.read(cx);
        let new = index.saturating_add(1);

        error!("down {}", new);
        self.state.scroll_to_reveal_item(index);
        self.cursor.update(cx, |e, v| {
            *e = new;
        });
        cx.notify()
    }
}
