use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow};
use adw::gtk::{Box, ListBox, Orientation, SelectionMode, SearchEntry, ScrolledWindow, Image, Stack};
use std::path::Path;
mod cache;
use cache::*;
mod desktopsearch;
use desktopsearch::*;
mod graphlib;
use graphlib::grapher::create_graph_pixbuf;
use fancy_regex::Regex;

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
            .build();

        let apps = getdesktopfiles();

        // encasing the list in a scrolling element
        let scrolled_window = ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(adw::gtk::PolicyType::Never)
            .vscrollbar_policy(adw::gtk::PolicyType::Automatic)
            .propagate_natural_height(true)
            .build();

        // graph goes here
        let graph = adw::gtk::Picture::builder()
            .vexpand(true)
            .hexpand(true)
            .visible(true)
            .can_shrink(true)          // shrink if widget is smaller than content
            .keep_aspect_ratio(true)
            .build();

        let stack = Stack::new();
        stack.add_titled(&scrolled_window, Some("list"), "List");
        stack.add_titled(&graph, Some("graph"), "Graph");
        stack.set_visible_child_name("list");
        stack.set_vexpand(true);
        stack.set_hexpand(true);
        stack.set_visible(true);

        let content = Box::new(Orientation::Vertical, 0);
        content.append(&search_entry);
        content.append(&stack);

        let list_clone = list.clone();
        let app_clone = app.clone();
        let stack_clone = stack.clone();
        let graph_clone = graph.clone();

        search_entry.set_property("search-delay", &1u32); // set debounce to 1 ms (from 150 default)

        search_entry.connect_search_changed(move |entry| {
            let text = entry.text().to_lowercase();

            // making the graph
            if text.contains('=') {
                // transform f(x,y) = g(x,y) into f(x,y)-g(x,y)
                let mut functozero = text.replace("=", "-(");
                functozero.push_str(")");
                // add multiplications signs for xy) followed by xy( or digit, and after a digit followed by xy(
                let re = Regex::new(r"[xy)](?=[xy(]|\d)|\d(?=[xy(])").unwrap();
                functozero = re.replace_all(&functozero, "$0*").to_string();

                let pix = unsafe { create_graph_pixbuf(graph.allocated_width(), graph.allocated_height(), 10, &functozero) };
                if let Some(pix) = &pix {
                    let texture = adw::gdk::Texture::for_pixbuf(pix);
                    graph_clone.set_paintable(Some(&texture));
                } else {
                    graph_clone.set_paintable(None::<&adw::gdk::Texture>);
                }
                stack_clone.set_visible_child_name("graph");
            }
            else {
                stack_clone.set_visible_child_name("list");
            }
            
            let sortedapps = reeval(&text, apps.clone());

            while let Some(child) = list_clone.first_child() {
                list_clone.remove(&child);
            }

            for (name, icon, exec, score) in &sortedapps {
                let row = ActionRow::builder()
                    .title(name.as_str())
                    // .subtitle(score.to_string().as_str())
                    .build();

                // make icons and pictures big
                // TODO: keep icon name and dont regenerate on every list rebuild
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
                    // clear not relevant launch flags
                    let cleaned = exec_clone
                    .replace("%U", "")
                    .replace("%u", "")
                    .replace("%f", "")
                    .replace("%F", "")
                    .replace("%i", "")
                    .replace("%c", "")
                    .replace("%k", "");
                    let _ = std::process::Command::new("sh").arg("-c").arg(&cleaned).spawn();
                    app_cloneception.quit();
                });
                row.set_height_request(74);
                list_clone.append(&row);
            }
            if let Some(first_row) = list_clone.first_child() {
                if stack_clone.visible_child_name() == Some("list".into()) {
                    list_clone.select_row(Some(&first_row.downcast::<adw::ActionRow>().unwrap()));
                }
            }
        });

        // React to pressing Enter
        let list_for_enter = list.clone();
        search_entry.connect_activate(move |_| {
            if let Some(selected_row) = list_for_enter.selected_row() {
                selected_row.activate();
            }
        });
        search_entry.set_height_request(50);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("werve")
            .default_width(768)
            .default_height(648)
            .decorated(false)
            .content(&content)
            .build();
            window.present();
    });

    application.run();
}