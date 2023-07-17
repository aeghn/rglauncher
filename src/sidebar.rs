use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};
use gio::{glib, prelude::{Cast, StaticType, CastNone}};
use glib::{BoxedAnyObject, clone, IsA, Receiver};


use gtk::{prelude::{FrameExt}, traits::BoxExt};
use gtk::traits::WidgetExt;
use tracing::error;


use crate::{plugins::{Plugin, PluginResult}, sidebar_row::SidebarRow};

pub struct Sidebar {
    pub scrolled_window: gtk::ScrolledWindow,
    pub list_view: gtk::ListView,
    pub selection_model: gtk::SingleSelection,
    pub list_store: gio::ListStore,
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
            .css_name("sidebar")
            .can_focus(false)
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .child(&list_view)
            .focusable(false)
            .build();

        Sidebar { scrolled_window, list_view,  selection_model, list_store }
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

        selection_model.connect_selected_item_notify(clone!(@strong view => move |selection| {
            let item = selection.selected_item();
            if let Some(boxed) = item {
                let tt = boxed.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();
                let preview = tt.preview();
                preview.set_halign(Center);
                preview.set_valign(Center);
                preview.set_hexpand(true);
                sidebar_scroll_window.set_child(Some(&preview));
            }
        }));

        selection_model
    }
}
