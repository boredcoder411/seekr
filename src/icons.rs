use std::io::Read;

use crate::resources;
use gtk::{
    gdk_pixbuf::Pixbuf,
    gio::{self, Icon},
};

pub fn load_image_resource(content: &[u8], size: (i32, i32)) -> Icon {
    let cursor = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(content));
    Icon::from(
        Pixbuf::from_stream_at_scale(
            &cursor,
            size.0,
            size.1,
            true,
            None::<&gtk::gio::Cancellable>,
        )
        .unwrap(),
    )
}

pub fn get_icon(name_or_path: &str) -> Icon {
    let mut res = resources::ICON_MAP.get().write().unwrap();
    if let Some(e) = res.get(name_or_path) {
        return e.clone();
    } else {
        if std::path::Path::new(name_or_path).exists() {
            if let Ok(mut f) = std::fs::File::open(std::path::Path::new(name_or_path)) {
                let mut buf = vec![];
                let _ = f.read_to_end(&mut buf);
                return load_image_resource(&buf, (512, 512));
            }
        }
        let icon = Icon::from(gio::ThemedIcon::from_names(&[name_or_path]));
        res.insert(name_or_path.to_string(), icon.clone());
        return icon;
    }
}
