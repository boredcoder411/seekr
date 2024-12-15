use crate::resources;
use gtk::gio;

pub fn get_icon(name: &str) -> gio::Icon {
    let mut res = resources::ICON_MAP.get().write().unwrap();
    if let Some(e) = res.get(name) {
        return e.clone();
    } else {
        let icon = gio::Icon::from(gio::ThemedIcon::from_names(&[name]));
        res.insert(name.to_string(), icon.clone());
        return icon;
    }
}
