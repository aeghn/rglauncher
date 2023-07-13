use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};
use gio::{glib, prelude::{Cast, StaticType, CastNone}};
use glib::{BoxedAnyObject, Receiver};


use gtk::{prelude::{FrameExt}, traits::BoxExt};
use gtk::traits::WidgetExt;
use tracing::error;


use crate::{plugins::{Plugin, PluginResult}, row::SidebarRow};
use crate::plugins::app::AppPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::windows::HyprWindows;
use crate::shared::UserInput;

pub struct Sidebar {
    pub list_view: gtk::ListView,
    pub scrolled_window: gtk::ScrolledWindow,
    pub selection_model: gtk::SingleSelection,
    pub list_store: gio::ListStore,
}

impl Sidebar {
    pub fn new() -> Self {
        let list_store = gio::ListStore::new(BoxedAnyObject::static_type());

        let sorter = gtk::CustomSorter::new(move |_o1, _o2| {
            let plugin_result1 =
                _o1.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();
            let plugin_result2 =
                _o2.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();

            plugin_result1.get_score().cmp(&plugin_result2.get_score()).into()
        });
        let sorted_model = gtk::SortListModel::new(None::<gio::ListModel>, Some(sorter));
        sorted_model.set_model(Some(&list_store));

        let selection_model = gtk::SingleSelection::new(Some(sorted_model.clone()));

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

        let list_view = gtk::ListView::new(Some(selection_model.clone()), Some(factory));
        list_view.add_css_class("sidebar");
        list_view.set_can_focus(false);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .child(&list_view)
            .focusable(false)
            .build();

        Sidebar { list_view, scrolled_window, selection_model, list_store }
    }
}
