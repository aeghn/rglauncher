use arc_swap::ArcSwap;
use chin_tools::AResult;
use fragile::Fragile;
use gio::{Icon, MemoryInputStream};

use glib::Bytes;
use gtk::gdk_pixbuf::Pixbuf;
use lazy_static::lazy_static;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;

lazy_static! {
    static ref ICON_PATHS: ArcSwap<Vec<PathBuf>> = ArcSwap::new(Arc::new(Vec::new()));
    static ref ICON_MAP: ArcSwap<HashMap<smol_str::SmolStr, Option<Arc<Fragile<Icon>>>>> =
        ArcSwap::new(Arc::new(HashMap::new()));
    static ref LOGO: Arc<Fragile<Icon>> = Arc::new(Fragile::new(Icon::from(
        load_from_svg(include_str!("../../../data/logo.svg")).unwrap()
    )));
}

fn load_from_svg(svg_data: &str) -> AResult<Pixbuf> {
    let image_stream = MemoryInputStream::from_bytes(&Bytes::from(svg_data.as_bytes()));

    Ok(Pixbuf::from_stream_at_scale(
        &image_stream,
        256,
        256,
        true,
        None::<&gio::Cancellable>,
    )?)
}

pub fn set_icon_dirs<T: AsRef<Path>>(paths: Vec<T>) {
    let paths = paths.iter().map(|e| e.as_ref().to_path_buf()).collect();

    ICON_PATHS.store(Arc::new(paths));
}

pub fn get_icon(name: &str) -> Arc<Fragile<Icon>> {
    if let Some(icon) = get_icon_inner(name) {
        icon
    } else {
        LOGO.clone()
    }
}

fn get_icon_inner(name: &str) -> Option<Arc<Fragile<Icon>>> {
    if let Some(icon) = ICON_MAP.load().get(name) {
        icon.clone()
    } else {
        let icon = ICON_PATHS
            .load()
            .iter()
            .filter_map(|e| {
                let svg_path = e.join(format!("{}.svg", name));
                if svg_path.exists() {
                    read_to_string(svg_path)
                        .ok()
                        .and_then(|e| load_from_svg(&e).ok())
                        .map(|e| Icon::from(e))
                } else {
                    None
                }
            })
            .nth(0);

        let icon = icon.clone().map(|e| Arc::new(Fragile::from(e)));
        let mut icon_map: HashMap<SmolStr, Option<Arc<Fragile<Icon>>>> = ICON_MAP
            .load()
            .iter()
            .map(|(key, icon)| (key.clone(), icon.clone()))
            .collect();
        icon_map.insert(name.into(), icon.clone());
        ICON_MAP.store(Arc::new(icon_map));

        return icon;
    }
}
