use gtk::glib;
use gtk::subclass::prelude::*;

use gtk::CompositeTemplate;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "sidebar_row.ui")]
pub struct SidebarRow {
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub content: TemplateChild<gtk::Frame>,
}

#[glib::object_subclass]
impl ObjectSubclass for SidebarRow {
    const NAME: &'static str = "SidebarRow";
    type Type = super::SidebarRow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for SidebarRow {}
impl WidgetImpl for SidebarRow {}
impl BoxImpl for SidebarRow {}
