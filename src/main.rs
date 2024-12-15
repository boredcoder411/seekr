use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use rust_i18n::t;
use search::SearchManager;

mod app;
mod bus;
mod conf;
mod icons;
mod locale;
mod resources;
mod search;
mod ui;

rust_i18n::i18n!("locales", fallback = "en");

async fn activate(
    current_css_provider: Arc<RwLock<gtk::CssProvider>>,
    config: Arc<RwLock<conf::Config>>,
    app: &Application,
) {
    let settings = gtk::Settings::default().expect("Failed to create GTK settings.");
    settings.set_gtk_icon_theme_name(Some(&config.read().await.general.theme));

    let rt = Runtime::new().expect("Unable to create Runtime");
    let window = ApplicationWindow::builder()
        .application(app)
        .title("seekr")
        .css_name("window")
        .resizable(false)
        .decorated(false)
        .hide_on_close(true)
        .build();

    if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        if xdg_current_desktop.to_lowercase() == "gnome" {
            window.add_css_class("gnome");
        }
    }

    window.set_default_size(600, -1);

    let (manager, (tomanager, frommanager)) = SearchManager::new();
    manager.manage();

    let entry = gtk::Entry::builder()
        .hexpand(true)
        .css_name("input")
        .activates_default(true)
        .placeholder_text(&config.read().await.general.search_placeholder)
        .build();

    let represent_action = gtk::gio::SimpleAction::new("represent", None);
    let conf_arc_clone = config.clone();
    represent_action.connect_activate(glib::clone!(
        #[weak]
        window,
        #[strong]
        tomanager,
        #[weak]
        entry,
        move |_, _| {
            let _ = tomanager.send(search::SearchEvent::Represent);
            let rt = Runtime::new().expect("Unable to create Runtime");
            rt.block_on(async {
                let mut conf = conf_arc_clone.write().await;
                let ccp = current_css_provider.read().await;
                conf.reload();
                entry.set_placeholder_text(Some(&conf.general.search_placeholder.clone()));
                let new_css_provider = load_css(conf.css.clone(), Some(ccp.clone()));
                drop(ccp);
                drop(conf);
                {
                    let mut ccp = current_css_provider.write().await;
                    *ccp = new_css_provider;
                }
            });
            window.present();
        }
    ));
    window.add_action(&represent_action);

    let input_container = gtk::Box::builder()
        .height_request(60)
        .hexpand(true)
        .spacing(5)
        .css_name("inputBox")
        .name("inputBox")
        .build();

    entry.connect_changed(glib::clone!(
        #[strong]
        tomanager,
        move |e| {
            let term = e.text().to_string();
            if !term.is_empty() {
                e.set_css_classes(&["has_input"])
            } else {
                e.set_css_classes(&[])
            }
            let _ = tomanager.send(search::SearchEvent::Term(term));
        }
    ));

    input_container.append(&entry);

    let shell = gtk::Box::builder()
        .hexpand(true)
        .vexpand(false)
        .name("shell")
        .css_name("shell")
        .orientation(gtk::Orientation::Vertical)
        .build();

    let scroll_container = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .height_request(405)
        .max_content_height(405)
        .min_content_height(0)
        .css_name("resultBox")
        .build();

    let result_box = gtk::Box::builder()
        .hexpand(true)
        .orientation(gtk::Orientation::Vertical)
        .build();

    scroll_container.set_child(Some(&result_box));
    #[allow(deprecated)]
    scroll_container.hide();

    shell.append(&input_container);
    shell.append(&scroll_container);
    window.set_child(Some(&shell));

    let clear_results = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
        move || {
            while let Some(child) = result_box.first_child() {
                result_box.remove(&child);
            }
            result_box.set_css_classes(&[]);
            #[allow(deprecated)]
            scroll_container.hide();
        }
    );

    let show_math = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
        move |res: f64| {
            #[allow(deprecated)]
            scroll_container.show();
            let math_box = gtk::Box::builder()
                .css_name("mathResult")
                .hexpand(true)
                .height_request(395)
                .orientation(gtk::Orientation::Vertical)
                .build();
            let head_box = gtk::Box::builder()
                .css_classes(["head"])
                .hexpand(true)
                .spacing(5)
                .halign(gtk::Align::Center)
                .build();
            let title = gtk::Label::builder()
                .hexpand(true)
                .halign(gtk::Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();

            title.set_text(&t!("expr_eval").to_string());
            let head_icon = gtk::Image::builder()
                .pixel_size(12)
                .gicon(&icons::get_icon("plus-symbolic"))
                .build();
            head_icon.set_css_classes(&["eval_icon"]);

            head_box.append(&head_icon);
            head_box.append(&title);

            let answer_box = gtk::Box::builder()
                .hexpand(true)
                .vexpand(true)
                .css_classes(["answer_box"])
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();

            let answer = gtk::Label::builder()
                .css_classes(["answer"])
                .halign(gtk::Align::Center)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            answer.set_text(&format!("{res}"));
            answer_box.append(&answer);

            math_box.append(&head_box);
            math_box.append(&answer_box);

            result_box.append(&math_box);
        }
    );

    let add_entries = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        tomanager,
        #[strong]
        scroll_container,
        move |entries: Vec<app::AppEntry>| {
            let entries_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(2)
                .build();
            if !entries.is_empty() {
                #[allow(deprecated)]
                scroll_container.show();
                let title = gtk::Label::builder()
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .css_name("title")
                    .build();

                title.set_label(&t!("apps").to_string());
                entries_box.append(&title);
            }

            let cfg = rt.block_on(async { config.clone().read().await.clone() });
            for entry in entries {
                let button = ui::EntryButton(&cfg, entry, &tomanager);
                entries_box.append(&button);
            }

            result_box.append(&entries_box);
        }
    );

    window.present();

    {
        glib::spawn_future_local(glib::clone!(async move {
            while let Ok(ev) = frommanager.recv().await {
                match ev {
                    search::ManagerEvent::DisplayEntries(entries) => add_entries(entries),
                    search::ManagerEvent::Mathematic(res) => show_math(res),
                    search::ManagerEvent::Clear => clear_results(),
                    search::ManagerEvent::Close => {
                        window.close();
                    }
                }
            }
        }));
    }
}

fn load_css(css: String, previous_provider: Option<gtk::CssProvider>) -> gtk::CssProvider {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);

    if let Some(previous_provider) = previous_provider {
        gtk::style_context_remove_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &previous_provider,
        );
    }

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    return provider;
}

fn main() {
    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();
    rust_i18n::set_locale(&locale::get_locale());
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let config = conf::Config::parse(conf::init_config_dir());
    if bus::app_is_running() {
        bus::send_represent_event();
    } else {
        gtk::init().expect("Unable to init gtk");
        let current_css_provider = load_css(config.css.clone(), None);

        let application = Application::new(Some(conf::APP_ID), Default::default());

        application.connect_activate(move |app| {
            rt.block_on(async {
                activate(
                    Arc::new(RwLock::new(current_css_provider.clone())),
                    Arc::new(RwLock::new(config.clone())),
                    app,
                )
                .await
            });
        });

        application.run();
    }
}
