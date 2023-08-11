use std::collections::HashMap;
use std::sync::{LockResult, Mutex};
use lazy_static::lazy_static;
use fragile::Fragile;
use gio::{AppInfo, Icon};
use gio::traits::AppInfoExt;
use glib::ObjectType;

lazy_static! {
    static ref ICON_MAP: Mutex<HashMap<String, Fragile<Icon>>> = Mutex::new(HashMap::new());
}

pub fn get_icon(name: &str) -> Icon {
    let mut guard = ICON_MAP.lock().unwrap();
    let oficon = guard.get(name);

    if let Some(ficon) = oficon {
        return ficon.get().clone()
    } else {
        if guard.len() == 0 {
            let mut oi: Option<Icon> = None;
            for app_info in AppInfo::all() {
                if let Some(icon) = app_info.icon() {
                    let iname = app_info.name().to_string();
                    if iname.eq(name) {
                        oi = Some(icon.clone());
                    }
                    guard.insert(iname, Fragile::from(icon));
                }
            }
            if let Some(icon) = oi {
                return icon;
            }
        }
        let _icon = gio::Icon::from(gio::ThemedIcon::from_names(&[name, ]));
        guard.insert(name.to_string(), Fragile::from(_icon.clone()));
        _icon
    }
}