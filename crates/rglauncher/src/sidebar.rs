use flume::Sender;
use gio::{
    glib,
    prelude::ListModelExt,
    prelude::{Cast, CastNone},
};
use glib::{object::IsA, variant::ToVariant, BoxedAnyObject, MainContext, Priority, StrV};
use gtk::{
    prelude::WidgetExt,
    prelude::{ListItemExt, SelectionModelExt},
};

use std::sync::Arc;

use backend::plugins::PluginResult;
use backend::ResultMsg;

use crate::sidebarrow::SidebarRow;

pub enum SidebarMsg {
    Result(Vec<Arc<dyn PluginResult>>),
    NextItem,
    PreviousItem,
    HeadItem,
    Enter,
}

#[derive(Clone)]
pub struct Sidebar {
    pub scrolled_window: gtk::ScrolledWindow,
    list_view: gtk::ListView,
    selection_model: gtk::SingleSelection,
    list_store: gio::ListStore,

    pub sidebar_sender: flume::Sender<SidebarMsg>,
    sidebar_receiver: flume::Receiver<SidebarMsg>,
}

impl Sidebar {
    pub fn new(
        result_sender: &Sender<ResultMsg>,
        sidebar_sender: flume::Sender<SidebarMsg>,
        sidebar_receiver: flume::Receiver<SidebarMsg>,
    ) -> Self {
        let list_store = gio::ListStore::new::<BoxedAnyObject>();
        let selection_model = Sidebar::build_selection_model(&list_store, result_sender);
        let factory = Sidebar::build_signal_list_item_factory();

        let list_view = gtk::ListView::builder()
            .factory(&factory)
            .model(&selection_model)
            .can_focus(false)
            .css_classes(["sidebar-view"])
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
            .css_classes(StrV::from(vec!["sidebar"]))
            .child(&list_view)
            .focusable(false)
            .can_focus(false)
            .width_request(300)
            .build();

        Sidebar {
            scrolled_window,
            list_view,
            selection_model,
            list_store,
            sidebar_sender,
            sidebar_receiver,
        }
    }

    fn handle_msg(&mut self, msg: SidebarMsg) {
        match msg {
            SidebarMsg::NextItem => {
                let new_selection = if self.selection_model.n_items() > 0 {
                    std::cmp::min(
                        self.selection_model.n_items() - 1,
                        self.selection_model.selected() + 1,
                    )
                } else {
                    0
                };
                self.scroll_to_item(&new_selection);
            }
            SidebarMsg::PreviousItem => {
                let new_selection = if self.selection_model.selected() > 0 {
                    self.selection_model.selected() - 1
                } else {
                    0
                };
                self.scroll_to_item(&new_selection);
            }
            SidebarMsg::HeadItem => {
                let new_selection = 0;
                self.scroll_to_item(&new_selection);
            }
            SidebarMsg::Enter => {
                let item = self.selection_model.selected_item();
                glib::idle_add_local_once(|| {
                    if let Some(boxed) = item {
                        let tt = boxed
                            .downcast_ref::<BoxedAnyObject>()
                            .unwrap()
                            .borrow::<Arc<dyn PluginResult>>();
                        tt.on_enter();
                    }
                });
            }
            SidebarMsg::Result(results) => {
                let list_store = self.list_store.clone();
                MainContext::ref_thread_default().spawn_local_with_priority(
                    Priority::LOW,
                    async move {
                        let boxed_objects: Vec<BoxedAnyObject> = results
                            .into_iter()
                            .map(|e| BoxedAnyObject::new(e))
                            .collect();
                        list_store.splice(0, list_store.n_items(), &boxed_objects);
                    },
                );
            }
        }
    }

    fn scroll_to_item(&mut self, new_selection: &u32) {
        self.selection_model
            .select_item(new_selection.clone(), true);
        self.list_view
            .activate_action("list.scroll-to-item", Some(&new_selection.to_variant()))
            .unwrap();
    }

    pub fn loop_recv(&mut self) {
        let mut sidebar = self.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            loop {
                match sidebar.sidebar_receiver.recv_async().await {
                    Ok(sidebar_msg) => {
                        sidebar.handle_msg(sidebar_msg);
                    }
                    Err(_) => {}
                }
            }
        });
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
            let plugin_result = plugin_result_box.borrow::<Arc<dyn PluginResult>>();

            let child = item.child().and_downcast::<SidebarRow>().unwrap();
            child.arrange_sidebar(plugin_result.as_ref());
        });

        factory.connect_unbind(move |_factory, item| {
            let _list_item = item.downcast_ref::<gtk::ListItem>().unwrap();

            let child = item.child().and_downcast::<SidebarRow>().unwrap();
            child.unbind_all();
        });

        factory
    }

    fn build_selection_model(
        list_model: &impl IsA<gio::ListModel>,
        selection_change_sender: &flume::Sender<ResultMsg>,
    ) -> gtk::SingleSelection {
        let selection_model = gtk::SingleSelection::builder().model(list_model).build();

        let result_sender = selection_change_sender.clone();

        selection_model.connect_selected_item_notify(move |selection| {
            result_sender
                .send(ResultMsg::ChangeSelect(selection.selected()))
                .expect("TODO: panic message");
        });

        selection_model
    }
}
