use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow};
use adw::gtk::{Box, ListBox, Orientation, SelectionMode, SearchEntry, ScrolledWindow};
use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use std::env::var;
mod cache;
use cache::*;


fn main() {
    let application = Application::builder()
        .application_id("dev.zmeyka.werve")
        .build();

    application.connect_activate(|app| {
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search...")
            .margin_top(20)
            .margin_end(20)
            .margin_bottom(0)
            .margin_start(20)
            .build();

        // React to text changes
        search_entry.connect_search_changed(|entry| {
            let _text = entry.text();
            println!("Updated text");
        });

        // React to pressing Enter
        search_entry.connect_activate(|entry| {
            let _text = entry.text();
            println!("Enter pressed");
        });
        search_entry.set_height_request(50);

        let list = ListBox::builder()
            .margin_top(20)
            .margin_end(20)
            .margin_bottom(20)
            .margin_start(20)
            .selection_mode(SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();

        let apps = getdesktopfiles();
        for (name, icon, exec, score) in apps {
            let row = ActionRow::builder()
                .title(name.as_str())
                .subtitle(exec.as_str())
                .build();
            list.append(&row);
        }

        // encasing the list in a scrolling element
        let scrolled_window = ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(adw::gtk::PolicyType::Never)
            .vscrollbar_policy(adw::gtk::PolicyType::Automatic)
            .propagate_natural_height(true)
            .build();

        let content = Box::new(Orientation::Vertical, 0);
        content.append(&search_entry);
        content.append(&scrolled_window);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("werve")
            .default_width(600)
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}



fn getdesktopfiles() -> Vec<(String, String, String, i32)> {
    let cdesktop_bind = var("XDG_CURRENT_DESKTOP").unwrap();
    let current_desktop = cdesktop_bind.as_str();
    let cache_count = read_cache();
    let mut desktopfiles = desktop_entries(&get_languages_from_env())
        .clone()
        .into_iter()
        .filter_map(|entry| {
            let desktop_group = entry.groups.0.get("Desktop Entry")?;
            if let Some((value, _)) = desktop_group.0.get("NoDisplay") {
                if value == "true" {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("Terminal") {
                if value == "true" {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("OnlyShowIn") {
                let envs: Vec<&str> = value.split(';').filter(|s| !s.is_empty()).collect();
                if !envs.contains(&current_desktop) {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("NotShowIn") {
                let envs: Vec<&str> = value.split(';').filter(|s| !s.is_empty()).collect();
                if envs.contains(&current_desktop) {
                    return None;
                }
            }
            let name = desktop_group.0.get("Name")?.0.clone();
            let icon = desktop_group.0.get("Icon")?.0.clone();
            let exec = desktop_group.0.get("Exec")?.0.clone();
            let score = cache_count.get(&name).unwrap_or(&0).clone();
            Some((name, icon, exec, score))
        }).collect::<Vec<_>>();

    desktopfiles.sort_by(|a, b| a.0.cmp(&b.0)); // 0 is name, lower means sorted alphabetically
    desktopfiles.sort_by(|a, b| b.3.cmp(&a.3)); // 3 is score, higher means more relevant

    // for item in &desktopfiles {
    //     println!("{:?}", item);
    // }

    desktopfiles
}