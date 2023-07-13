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
        let app_plugin = AppPlugin::new();
        let window_plugin = HyprWindows::new();
        let clip_plugin = ClipboardPlugin::new("/home/chin/.cache/rglauncher/store.db");
        let mut plugins : Vec<Box<dyn Plugin + Send>> = vec![];
        plugins.push(Box::new(app_plugin));
        plugins.push(Box::new(window_plugin));
        plugins.push(Box::new(clip_plugin));

        let list_store = gio::ListStore::new(BoxedAnyObject::static_type());

        let sorter = gtk::CustomSorter::new(move |_o1, _o2| {
            let plugin_result1 =
                _o1.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult + Send>>();
            let plugin_result2 =
                _o2.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult + Send>>();

            plugin_result1.get_score().cmp(&plugin_result2.get_score()).into()
        });
        let sorted_model = gtk::SortListModel::new(Some(list_store.clone()), Some(sorter));

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
            let plugin_result = plugin_result_box.borrow::<Box<dyn PluginResult + Send>>();

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

        // {
        //     let list_store = list_store.clone();
        //     inputrx.attach(None, move |e| {
        //         let user_input = UserInput::new(e.as_str());
        //
        //         list_store.remove_all();
        //         plugins.iter().for_each(|plugin| {
        //             let results = plugin.handle_input(&user_input);
        //             for x in results {
        //                 list_store.append(&BoxedAnyObject::new(x));
        //             }
        //         });
        //
        //         glib::Continue(true)
        //     });
        // }

        Sidebar { list_view, scrolled_window, selection_model, list_store }
    }
}
