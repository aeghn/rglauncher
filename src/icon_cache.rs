pub struct IconCache {
    icon_map: HashMap<String, gio::Icon>
}

impl IconCache {
    pub fn new() -> Self {
        IconCache {icon_map: HashMap::new()}
    }

    pub fn get_gicon_by_name(&mut name: &str) -> gio::Icon {
        if let Some
    } 
}
