use gtk::prelude::*;

pub fn item(label: &str, action: &str) -> gio::MenuItem {
    gio::MenuItem::new(Some(&label), Some(&action))
}

pub fn item_p<P: ToVariant + FromVariant>(label: &str, action: &str, param: P) -> gio::MenuItem {
    let item = gio::MenuItem::new(Some(&label), None);
    item.set_action_and_target_value(Some(&action), Some(&param.to_variant()));
    item
}

pub fn section<I: IntoIterator<Item = gio::MenuItem>>(label: Option<&str>, items: I) -> gio::MenuItem {
    let menu = gio::Menu::new();
    for item in items {
        menu.append_item(&item);
    }

    gio::MenuItem::new_section(label, &menu)
}

pub fn submenu<I: IntoIterator<Item = gio::MenuItem>>(label: Option<&str>, items: I) -> gio::MenuItem {
    let menu = gio::Menu::new();
    for item in items {
        menu.append_item(&item);
    }

    gio::MenuItem::new_submenu(label, &menu)
}

pub fn menu<I: IntoIterator<Item = gio::MenuItem>>(items: I) -> gio::Menu {
    let menu = gio::Menu::new();
    for item in items {
        menu.append_item(&item);
    }
    menu
}