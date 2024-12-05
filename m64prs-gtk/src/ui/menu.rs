pub fn load_menu() -> gio::MenuModel {
    const UI_XML: &str = gtk::gtk4_macros::include_blueprint!("src/ui/menu.blp");
    gtk::Builder::from_string(UI_XML)
        .object::<gio::MenuModel>("root")
        .expect("menu.blp should contain object `root`")
}
