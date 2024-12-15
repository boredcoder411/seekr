use fragile::Fragile;
use gtk::gio::Icon;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// inspired from https://github.com/aeghn/rglauncher/blob/2789af0c36f5929a448807584aaaf57685162891/crates/rglauncher-gtk/src/iconcache.rs#L12
lazy_static! {
    pub static ref ICON_MAP: Arc<Fragile<RwLock<HashMap<String, Icon>>>> =
        Arc::new(Fragile::new(RwLock::new(HashMap::new())));
}
