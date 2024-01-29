use fragile::Fragile;
use gio::{Icon, MemoryInputStream};

use crate::constants;
use glib::Bytes;
use gtk::gdk_pixbuf::Pixbuf;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref ICON_MAP: Mutex<HashMap<String, Arc<Fragile<Icon>>>> = Mutex::new(HashMap::new());
    static ref LOGO_ICON: Arc<Fragile<Icon>> = Arc::new(Fragile::new(Icon::from(load_from_svg(
        include_str!("../../../res/logo.svg")
    ))));
}

fn load_from_svg(s: &str) -> Pixbuf {
    let logo_bytes = Bytes::from(s.as_bytes());
    let image_stream = MemoryInputStream::from_bytes(&logo_bytes);
    Pixbuf::from_stream_at_scale(&image_stream, 256, 256, true, None::<&gio::Cancellable>)
        .expect("unablt to read dom stream")
}

pub fn get_icon(name: &str) -> Arc<Fragile<Icon>> {
    let name = icon_name_map(name);
    let mut guard = ICON_MAP.lock().unwrap();
    let oficon = guard.get(name);
    if name == "" || name == constants::APP_ID {
        return get_logo();
    }

    if let Some(ficon) = oficon {
        return ficon.clone();
    } else {
        let _icon = gio::Icon::from(gio::ThemedIcon::from_names(&[name]));

        let arc = Arc::new(Fragile::from(_icon.clone()));
        guard.insert(name.to_string(), arc.clone());
        return arc;
    }
}

pub fn get_logo() -> Arc<Fragile<Icon>> {
    LOGO_ICON.clone()
}

pub fn icon_name_map(name: &str) -> &str {
    match name {
        "jetbrains-studio" => "android-studio",
        "code-url-handler" => "visual-studio-code",
        _ => name
    }
}
