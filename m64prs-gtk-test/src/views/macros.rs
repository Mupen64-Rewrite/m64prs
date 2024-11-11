//! Utility macros for building UIs.
//! 
//! These create more declarative syntax for declaring UIs.

#[doc(hidden)]
mod sealed {
    trait Sealed {}
}

pub(in crate::views) mod menu {
    /// Constructs a root menu.
    /// ```ignore
    /// menu::root!([name] { contents });
    /// ```
    /// - **name**: name for the root menu.
    macro_rules! root {
        ([$name:ident] $contents:block) => {
            {
                let $name = ::gtk::gio::Menu::new();
                $contents
                $name.upcast()
            }
        };
    }
    /// Constructs a section.
    /// ```ignore
    /// // no section label
    /// menu::section!([parent, child] { contents });
    /// // with section label
    /// menu::section!([parent, child] "label" { contents });
    /// ```
    /// - **parent** (var): the parent menu.
    /// - **child** (var): name for the section.
    /// - **label**: label to display on the section.
    macro_rules! section {
        ([$parent:ident, $child:ident] $contents:block) => {
            $parent.append_section(None, &{
                let $child = ::gtk::gio::Menu::new();
                $contents
                $child
            })
        };
        ([$parent:ident, $child:ident] $label:literal $contents:block) => {
            $parent.append_section(Some($label), &{
                let $child = ::gtk::gio::Menu::new();
                $contents
                $child
            })
        };
    }
    /// Constructs a submenu.
    /// ```ignore
    /// menu::submenu!([parent, child] "label" { contents });
    /// ```
    /// - **parent** (var): the parent menu.
    /// - **child** (var): name for the child submenu.
    /// - **label** (string): label to display on the menu.
    macro_rules! submenu {
        ([$parent:ident, $child:ident] $label:literal $contents:block) => {
            $parent.append_submenu(Some($label), &{
                let $child = ::gtk::gio::Menu::new();
                $contents
                $child
            })
        };
    }
    /// Constructs a menu item.
    /// ```ignore
    /// menu::item!([parent] "label" => action)
    /// ```
    /// - **parent** (var): variable for the parent menu.
    /// - **label** (string): label to display on the item.
    /// - **action** (expr): action to associate with the item.
    macro_rules! item {
        ([$parent:ident] $label:literal => $action:expr) => {
            $parent.append_item(&::gtk::gio::MenuItem::new(Some($label), Some($action)));
        };
        ([$parent:ident] $label:literal => [$param:expr] $action:expr) => {
            $parent.append_item(&{
                let menu_item = ::gtk::gio::MenuItem::new(Some($label), None);
                menu_item
                    .set_action_and_target_value(Some($action), Some(&::glib::Variant::from($param)));
                menu_item
            });
        };
    }

    pub(in crate::views) use item;
    pub(in crate::views) use section;
    pub(in crate::views) use submenu;
    pub(in crate::views) use root;
}

pub(in crate::views) mod action {
    macro_rules! simple {
        ($parent:ident[$name:expr] => |$action:pat_param| $contents:expr) => {
            $parent.add_action(&{
                let action = ::gtk::gio::SimpleAction::new($name, None);
                action.connect_activate(|$action, _| $contents);
                action
            });
        };
        ($parent:ident[$name:expr]<$ptype:ty> => |$action:pat_param, $param:pat_param| $contents:expr) => {
            $parent.add_action(&{
                let action = ::gtk::gio::SimpleAction::new(
                    $name, <$ptype as ::gtk::glib::Variant::StaticVariantType>::static_variant_type());
                action.connect_activate(|$action, $param| $contents);
                action
            });
        };
        ($parent:ident[$name:expr] => async |$action:pat_param| $contents:expr) => {
            $parent.add_action(&{
                let action = ::gtk::gio::SimpleAction::new($name, None);
                action.connect_activate(|$action, _| {
                    ::gtk::glib::spawn_future_local(async { $contents });
                });
                action
            });
        };
        ($parent:ident[$name:expr]<$ptype:ty> => async |$action:pat_param, $param:pat_param| $contents:expr) => {
            $parent.add_action(&{
                let action = ::gtk::gio::SimpleAction::new(
                    $name, <$ptype as ::gtk::glib::Variant::StaticVariantType>::static_variant_type());
                action.connect_activate(|$action, $param| {
                    ::gtk::glib::spawn_future_local(async { $contents });
                });
                action
            });
        }
    }

    pub(in crate::views) use simple;
}

/// 
macro_rules! take_owner {
    ($child:ident -> $parent:ident) => {
        {
            {
                #[inline(always)]
                fn check_valid<T: ::gtk::glib::prelude::ObjectType>(_: &T) {}
                check_valid(&$child);
                check_valid(&$parent);
            }
            $parent.add_weak_ref_notify_local(move || {
                drop($child);
            });
        };
    };
    (clone $child:ident -> $parent:ident) => {
        {
            {
                #[inline(always)]
                fn check_valid<T: ::gtk::glib::prelude::ObjectType>(_: &T) {}
                check_valid(&$child);
                check_valid(&$parent);
            }
            let $child = $child.clone();
            $parent.add_weak_ref_notify_local(move || {
                drop($child);
            });
        };
    }
}

pub(in crate::views) use take_owner;