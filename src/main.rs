use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow};
use adw::gtk::{Box, ListBox, Orientation, SelectionMode, SearchEntry, ScrolledWindow, Image};
use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use adw::gio::{File, FileIcon};
use std::env::var;
use std::path::Path;
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

        let list = ListBox::builder()
            .margin_top(20)
            .margin_end(20)
            .margin_bottom(20)
            .margin_start(20)
            .selection_mode(SelectionMode::Single)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();

        let apps = getdesktopfiles();

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

        let list_clone = list.clone();
        let app_clone = app.clone();

        search_entry.set_property("search-delay", &1u32); // set debounce to 1 ms (from 150 default)

        search_entry.connect_search_changed(move |entry| {
            let text = entry.text();
            
            let sortedapps = reeval(&text, apps.clone());

            while let Some(child) = list_clone.first_child() {
                list_clone.remove(&child);
            }

            for (name, icon, exec, score) in &sortedapps {
                let row = ActionRow::builder()
                    .title(name.as_str())
                    .subtitle(score.to_string().as_str())
                    .build();

                if Path::new(icon).is_absolute() && Path::new(icon).exists() {
                    let image = Image::from_file(icon);
                    image.set_pixel_size(48);
                    row.add_prefix(&image);
                } else if !icon.is_empty() {
                    let image = Image::from_icon_name(icon);
                    image.set_pixel_size(48);
                    row.add_prefix(&image);
                }

                let exec_clone = exec.clone();
                let name_clone = name.clone();
                let app_cloneception = app_clone.clone();
                row.set_activatable(true);
                row.connect_activated(move |_| {
                    println!("Detected: {}", exec_clone);
                    update_cache(&name_clone);
                    let cleaned = exec_clone
                    .replace("%U", "")
                    .replace("%u", "")
                    .replace("%f", "")
                    .replace("%F", "")
                    .replace("%i", "")
                    .replace("%c", "")
                    .replace("%k", "");
                    std::process::Command::new("sh").arg("-c").arg(&cleaned).spawn();
                    app_cloneception.quit();
                });
                list_clone.append(&row);
            }
            if let Some(first_row) = list_clone.first_child() {
                list_clone.select_row(Some(&first_row.downcast::<adw::ActionRow>().unwrap()));
            }
        });

        // React to pressing Enter
        let list_for_enter = list.clone();
        search_entry.connect_activate(move |_| {
            if let Some(selected_row) = list_for_enter.selected_row() {
                selected_row.activate(); // This triggers the row's `connect_activated` closure
            }
        });
        search_entry.set_height_request(50);

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


fn reeval(searchterm: &str, desktopfiles: Vec<(String, String, String, i32)>) -> Vec<(String, String, String, i32)> {
    if searchterm.is_empty() {
        return desktopfiles;
    }
    let mut desktopfiles2 = desktopfiles.into_iter().map(|entry| reeval_single(searchterm.to_lowercase(), entry)).collect::<Vec<_>>();
    desktopfiles2.sort_by(|a, b| a.3.cmp(&b.3)); // 3 is score, lower means more relevant
    desktopfiles2
}


fn reeval_single(searchterm: &str, toevaluate: (String, String, String, i32)) -> (String, String, String, i32) {
    let mut finalmult = 0;
    let mut find_from = 0;
    let mut found_first = false;
    let mut score:i32 = 1;
    let searchedstring = toevaluate.0.clone().to_lowercase();
    for letter in searchterm.chars() {
        let count = searchedstring.match_indices(letter).find_map(|(i, _)| (i >= find_from).then(|| i));
        if !found_first {
            finalmult += 1;
            if !count.is_none() {
                found_first = true;
                score += (count.unwrap() - find_from) as i32;
            }
            else {
                score *= 4;
            }
        }
        else {
            if !count.is_none(){
                score += (count.unwrap() - find_from).pow(2) as i32;
            }
            else {
                score *= 4;
            }
        }
        find_from += 1;
    }
    score = score * finalmult;
    (toevaluate.0, toevaluate.1, toevaluate.2, score)
}