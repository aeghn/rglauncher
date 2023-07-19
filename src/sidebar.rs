use std::borrow::Borrow;
use flume::RecvError;

use gio::{glib, prelude::{Cast, StaticType, CastNone}};
use glib::{BoxedAnyObject, Continue, IsA, StrV};


use gtk::{prelude::{FrameExt}};
use gtk::ResponseType::No;
use gtk::traits::WidgetExt;
use tracing::error;

use crate::{plugins::{PluginResult}, sidebar_row::SidebarRow};
use crate::shared::UserInput;

pub enum SidebarMsg {
    TextChanged(String),
    PluginResult(UserInput, Vec<Box<dyn PluginResult>>)
}

pub struct Sidebar {
    pub scrolled_window: gtk::ScrolledWindow,
    pub list_view: gtk::ListView,
    pub selection_model: gtk::SingleSelection,
    pub list_store: gio::ListStore,
    pub sorted_store: gtk::SortListModel,
    pub plugin_result_receiver: flume::Receiver<SidebarMsg>,
    pub plugin_result_sender: flume::Sender<SidebarMsg>,
    pub current_input: Option<UserInput>
}

impl Sidebar {
    pub fn new() -> Self {
        let list_store = gio::ListStore::new(BoxedAnyObject::static_type());
        let sorted_model = Sidebar::build_sorted_model(&list_store);
        let selection_model = Sidebar::build_selection_model(&sorted_model);
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
            .build();

        let (tx, rx) = flume::unbounded();



        Sidebar {
            scrolled_window,
            list_view,
            selection_model,
            list_store,
            sorted_store: sorted_model,
            plugin_result_receiver: rx,
            plugin_result_sender: tx,
            current_input: None
        }
    }

    pub async fn receive_msgs(&mut self) {
        let prr = &self.plugin_result_receiver;
        loop {
            let pr = prr.recv_async().await;
            match pr {
                Ok(msg) => {
                    match msg {
                        SidebarMsg::TextChanged(text) => {
                            self.current_input.replace(UserInput::new(text.as_str()));
                            error!("start remove");
                            self.list_store.remove_all();
                            error!("end remove");
                        }
                        SidebarMsg::PluginResult(ui_, pr_) => {
                            if let Some(ui) = &self.current_input {
                                if ui_.input == ui.input {
                                    error!("start extend");
                                    self.list_store.extend(pr_.into_iter()
                                        .map(move |e| {
                                            BoxedAnyObject::new(e)
                                        }));
                                    error!("end extend");
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
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

    fn build_signal_list_item_factory() -> gtk::SignalListItemFactory {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = SidebarRow::new();
            item.set_child(Some(&row));
        });

        factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let plugin_result_box = item.item().and_downcast::<BoxedAnyObject>().unwrap();
            let plugin_result = plugin_result_box.borrow::<Box<dyn PluginResult>>();

            let child = item.child().and_downcast::<SidebarRow>().unwrap();
            child.set_sidebar(plugin_result.as_ref());
        });

        factory
    }

    fn build_selection_model(list_model: &impl IsA<gio::ListModel>) -> gtk::SingleSelection {
        let selection_model = gtk::SingleSelection::builder()
            .model(list_model)
            .build();

        // selection_model.connect_selected_item_notify(clone!(@strong view => move |selection| {
        //     let item = selection.selected_item();
        //     if let Some(boxed) = item {
        //         let tt = boxed.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();
        //         let preview = tt.preview();
        //         preview.set_halign(Center);
        //         preview.set_valign(Center);
        //         preview.set_hexpand(true);
        //         sidebar_scroll_window.set_child(Some(&preview));
        //     }
        // }));

        selection_model
    }
}
