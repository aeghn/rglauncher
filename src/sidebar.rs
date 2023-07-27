use std::borrow::Borrow;
use std::ops::Deref;
use std::sync::Arc;
use flume::{Receiver, Sender, unbounded};
use futures::StreamExt;
use futures::select;
use gio::{glib, prelude::{Cast, StaticType, CastNone}};
use glib::{BoxedAnyObject, idle_add, IsA, StrV};


use gtk::{Image, Label, prelude::{FrameExt}};
use gtk::ResponseType::No;


use gtk::traits::{GridExt, WidgetExt};
use tracing::error;

use crate::{plugins::{PluginResult}};
use crate::inputbar::InputMessage;
use crate::shared::UserInput;
use crate::sidebar_row::SidebarRow;

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

    input_broadcast: async_broadcast::Receiver<Arc<InputMessage>>,
    sidebar_receiver: Receiver<SidebarMsg>,
    pub sidebar_sender: Sender<SidebarMsg>,
    selection_change_sender: Sender<BoxedAnyObject>,
}

impl Sidebar {
    pub fn new(input_broadcast: async_broadcast::Receiver<Arc<InputMessage>>,
               selection_change_sender: Sender<BoxedAnyObject>) -> Self {
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();

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
            selection_change_sender,
        }
    }

    pub async fn loop_recv(&mut self) {
        let sidebar_receiver = &self.sidebar_receiver;
        let mut input_receiver = self.input_broadcast.clone();
        loop {
            select! {
                sidebar_msg = sidebar_receiver.recv_async() => {
                    match sidebar_msg {
                        Ok(SidebarMsg::PluginResult(ui_, pr_)) => {
                            if let Some(ui) = &self.input {
                                if ui_.input == ui.input {
                                    self.list_store.append(&BoxedAnyObject::new(pr_));

                                }
                            }
                        }
                        _ => {}
                    }
                }
                input_msg = input_receiver.next() => {
                    match input_msg {
                        Some(arc) => {
                            match arc.borrow() {
                                InputMessage::TextChanged(text) => {
                                    self.input.replace(UserInput::new(text.as_str()));
                                    self.list_store.remove_all();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
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

    fn get_sidebar_item() -> SidebarRow {
        SidebarRow::new()
    }

    fn arrange_sidebar_item(grid: &SidebarRow, pr: &dyn PluginResult) {
        grid.set_sidebar(pr);
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

            let child = item.child().and_downcast::<SidebarRow>().unwrap();
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
