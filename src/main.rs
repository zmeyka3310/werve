use adw::prelude::*;

use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use adw::gtk::{Box, ListBox, Orientation, SelectionMode, SearchEntry};

fn main() {
    let application = Application::builder()
        .application_id("dev.zmeyka.werve")
        .build();

    application.connect_activate(|app| {
        // Create a search entry
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search...")
            .build();

        // React to text changes
        search_entry.connect_search_changed(|entry| {
            let text = entry.text();
            println!("Updated text");
        });

        // Optionally react to pressing Enter
        search_entry.connect_activate(|entry| {
            let text = entry.text();
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
        list.append(&search_entry);

        // Combine the content in a box
        let content = Box::new(Orientation::Vertical, 0);
        // Adwaitas' ApplicationWindow does not include a HeaderBar
        // content.append(&HeaderBar::new());
        content.append(&list);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("werve")
            .default_width(600)
            // add content to window
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}