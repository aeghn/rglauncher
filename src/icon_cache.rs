use std::collections::HashMap;
use std::sync::{Arc, LockResult, Mutex};
use lazy_static::lazy_static;
use fragile::Fragile;
use gio::{AppInfo, Icon};
use gio::traits::AppInfoExt;
use glib::ObjectType;

lazy_static! {
    static ref ICON_MAP: Mutex<HashMap<String, Arc<Fragile<Icon>>>> = Mutex::new(HashMap::new());
}

pub fn get_icon(name: &str) -> Arc<Fragile<Icon>> {
    let mut guard = ICON_MAP.lock().unwrap();
    let oficon  = guard.get(name);

    if let Some(ficon) = oficon {
        return ficon.clone()
    } else {
        if guard.len() == 0 {
            let mut oi: Option<Arc<Fragile<Icon>>> = None;
            for app_info in AppInfo::all() {
                if let Some(icon) = app_info.icon() {
                    let iname = app_info.name().to_string();
                    let arc = Arc::new(Fragile::from(icon));
                    if iname.eq(name) {
                        oi = Some(arc.clone());
                    }
                    guard.insert(iname, arc);
                }
            }
            if let Some(icon) = oi {
                return icon;
            }
        }
        let _icon = gio::Icon::from(gio::ThemedIcon::from_names(&[name, ]));
        let arc = Arc::new(Fragile::from(_icon.clone()));
        guard.insert(name.to_string(), arc.clone());
        arc
    }
}