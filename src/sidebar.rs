use std::borrow::Borrow;
use std::ops::Deref;
use flume::{Receiver, Sender, unbounded};


use gio::{glib, prelude::{Cast, StaticType, CastNone}};
use glib::{BoxedAnyObject, IsA, StrV};


use gtk::{Image, Label, prelude::{FrameExt}};
use gtk::ResponseType::No;


use gtk::traits::{GridExt, WidgetExt};
use tracing::error;

use crate::{plugins::{PluginResult}};
use crate::inputbar::InputMessage;
use crate::shared::UserInput;

pub enum SidebarMsg {
    TextChanged(String),
    PluginResult(UserInput, Box<dyn PluginResult>),
}

#[derive(Clone)]
pub struct Sidebar {
    pub scrolled_window: gtk::ScrolledWindow,
    list_view: gtk::ListView,
    selection_model: gtk::SingleSelection,
    sorted_model: gtk::SortListModel,
    list_store: gio::ListStore,

    input: Option<UserInput>,

    input_broadcast: barrage::Receiver<InputMessage>,
    sidebar_receiver: Receiver<SidebarMsg>,
    pub sidebar_sender: Sender<SidebarMsg>,
    pub selection_change_receiver: Receiver<BoxedAnyObject>,
    selection_change_sender: Sender<BoxedAnyObject>,
}

impl Sidebar {
    pub fn new(input_broadcast: barrage::Receiver<InputMessage>) -> Self {
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();
        let (selection_change_sender, selection_change_receiver) = unbounded();

        let list_store = gio::ListStore::new(BoxedAnyObject::static_type());
        let sorted_model = Sidebar::build_sorted_model(&list_store);
        let selection_model = Sidebar::build_selection_model(&sorted_model, &selection_change_sender);
        let factory = Sidebar::build_signal_list_item_factory();

        let list_view = gtk::ListView::builder()
            .factory(&factory)
            .model(&selection_model)
            .can_focus(false)
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .css_classes(StrV::from(vec!["sidebar"]))
            .child(&list_view)
            .focusable(false)
            .can_focus(false)
            .build();

        Sidebar {
            scrolled_window,
            list_view,
            selection_model,
            list_store,
            sorted_model,
            input: None,
            input_broadcast,
            sidebar_receiver,
            sidebar_sender,
            selection_change_receiver,
            selection_change_sender,
        }
    }

    pub async fn loop_recv(&self) {
        let prr = &self.sidebar_receiver;
        loop {
            if let Ok(msg) = prr.recv_async().await {
                match msg {
                    SidebarMsg::PluginResult(ui_, pr_) => {
                        if let Some(ui) = &self.input {
                            if ui_.input == ui.input {
                                self.list_store.append(&BoxedAnyObject::new(pr_));

                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn loop_recv_input(&mut self) {
        let broadcast = self.input_broadcast.clone();
        loop {
            if let Ok(msg) = broadcast.recv_async().await {
                error!("input: {:?}", msg);

                match msg {
                    InputMessage::TextChanged(text) => {
                        self.input.replace(UserInput::new(text.as_str()));

                        self.list_store.remove_all();
                    }
                    InputMessage::EmitSubmit(_) => {}
                }
            }
        }
    }

    fn build_sorted_model(list_model: &impl IsA<gio::ListModel>) -> gtk::SortListModel {
        let sorter = gtk::CustomSorter::new(move |item1, item2| {
            let plugin_result1 =
                item1.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();
            let plugin_result2 =
                item2.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();

            plugin_result1.get_score().cmp(&plugin_result2.get_score()).into()
        });

        gtk::SortListModel::builder()
            .model(list_model)
            .sorter(&sorter)
            .build()
    }

    fn get_sidebar_item() -> gtk::Grid {
        let grid = gtk::Grid::builder()
            .hexpand(true)
            .focusable(false)
            .can_focus(false)
            .build();

        let image = gtk::Image::builder()
            .pixel_size(48)
            .build();

        let label = crate::util::widget_utils::get_wrapped_label("", 0.5);
        grid.attach(&image, 0, 0, 1, 2);
        grid.attach(&label, 1, 0, 1, 1);

        grid
    }

    fn arrange_sidebar_item(grid: &gtk::Grid, pr: &dyn PluginResult) {
        if let Some(gi) = grid.child_at(0, 0) {
            if let Some(icon) = pr.sidebar_icon() {
                let image = gi.downcast_ref::<Image>().unwrap();
                image.set_from_gicon(&icon);
            }
        }

        if let Some(gi) = grid.child_at(1, 0) {
            if let Some(text) = pr.sidebar_label() {
                let label = gi.downcast_ref::<Label>().unwrap();
                label.set_label(text.as_str());
            }
        }

        if let Some(con) = pr.sidebar_content() {
            grid.attach(&con, 1, 1, 1, 1);
        }
    }

    fn build_signal_list_item_factory() -> gtk::SignalListItemFactory {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = Sidebar::get_sidebar_item();
            item.set_child(Some(&row));
        });

        factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let plugin_result_box = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let plugin_result = plugin_result_box.borrow::<Box<dyn PluginResult>>();

            let child = item.child().and_downcast::<gtk::Grid>().unwrap();
            Sidebar::arrange_sidebar_item(&child, plugin_result.as_ref())
        });

        factory
    }

    fn build_selection_model(list_model: &impl IsA<gio::ListModel>,
                             selection_change_sender: &Sender<BoxedAnyObject>)
                             -> gtk::SingleSelection {
        let selection_model = gtk::SingleSelection::builder()
            .model(list_model)
            .build();

        let selection_change_sender = selection_change_sender.clone();
        selection_model.connect_selected_item_notify(move |selection| {
            let item = selection.selected_item();
            if let Some(boxed) = item {
                let plugin_result_box = boxed.downcast::<BoxedAnyObject>()
                    .unwrap();
                selection_change_sender.send(plugin_result_box.clone())
                    .expect("Unable to send to preview");
            }
        });

        selection_model
    }
}
