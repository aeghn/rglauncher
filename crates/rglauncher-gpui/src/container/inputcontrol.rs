use crate::components::input::*;
use crate::plugindispatcher::PluginDispatcherMsg;
use crate::theme::*;
use flume::Sender;
use gpui::*;
use tracing::error;

pub struct InputControl {
    text_input: View<TextInput>,
}

impl InputControl {
    pub fn new(cx: &mut WindowContext, sender: &Sender<PluginDispatcherMsg>) -> View<Self> {
        let text_input = cx.new_view(|cx| TextInput {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "Type Something".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        });

        let sender = sender.clone();
        cx.subscribe(&text_input, move |_, event: &TextInputEvent, _| {
            if let TextInputEvent::Text(text) = event {
                if let Err(err) = sender.send(PluginDispatcherMsg::Filter(text.clone())) {
                    error!("unable to send from input control: {}, {}", text, err);
                }
            }
        })
        .detach();


        cx.new_view(|cx| InputControl { text_input })
    }
}

impl Render for InputControl {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let input = div()
            .flex()
            .flex_grow()
            .p_1()
            .rounded_md()
            .bg(theme.mantle)
            .border_1()
            .border_color(theme.crust)
            .child(self.text_input.clone());

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div().flex().gap_1().mt(px(10.)).child(input))
    }
}
