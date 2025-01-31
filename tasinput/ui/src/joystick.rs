mod inner {
    use std::{cell::Cell, sync::LazyLock};

    use graphene::Vec2;
    use gtk::{prelude::*, subclass::prelude::*};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Joystick)]
    pub struct Joystick {
        #[property(get, set, minimum = -128, maximum = 127)]
        pos_x: Cell<i8>,
        #[property(get, set, minimum = -128, maximum = 127)]
        pos_y: Cell<i8>,

        drag_pos: Cell<graphene::Point>,
    }

    impl Joystick {}

    #[glib::object_subclass]
    impl ObjectSubclass for Joystick {
        const NAME: &'static str = "TasDiJoystick";
        type Type = super::Joystick;
        type ParentType = gtk::Widget;

        fn class_init(class: &mut Self::Class) {
            class.set_css_name("td-joystick");
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Joystick {
        fn constructed(&self) {
            // let update_mod = ;
            let update_joy_pos = move |this: &super::Joystick, x: f64, y: f64| {
                // start + (x, y) = current pos
                let pos = graphene::Point::from_vec2(
                    &this
                        .imp()
                        .drag_pos
                        .get()
                        .to_vec2()
                        .add(&Vec2::new(x as f32, y as f32)),
                );
                let width = this.width() as f32;
                let height = this.height() as f32;

                const DEADZONE: i8 = 8;
                // convert mouse position to joystick position, accounting for deadzones
                let (mut jx, mut jy) = widget_to_js(pos, width, height);
                if -DEADZONE < jx && jx < DEADZONE {
                    jx = 0;
                }
                if -DEADZONE < jy && jy < DEADZONE {
                    jy = 0;
                }

                this.set_pos_x(jx);
                this.set_pos_y(jy);
            };
            let update_scroll =
                |this: &super::Joystick, ct: &gtk::EventController, dx: f64, dy: f64| {
                    const SENSITIVITY: f64 = 1.0;

                    let kmod = ct.current_event_state();
                    let ctrl = kmod.contains(gdk::ModifierType::CONTROL_MASK);
                    let shift = kmod.contains(gdk::ModifierType::SHIFT_MASK);

                    let (mut joy_x, mut joy_y) = (this.pos_x(), this.pos_y());
                    if ctrl && !shift {
                        joy_x = joy_x.saturating_add((dy * SENSITIVITY) as i8);
                    } else {
                        joy_x = joy_x.saturating_add((dx * SENSITIVITY) as i8);
                        joy_y = joy_y.saturating_add((-dy * SENSITIVITY) as i8);
                    }

                    this.set_pos_x(joy_x);
                    this.set_pos_y(joy_y);
                };

            let ct_drag = gtk::GestureDrag::new();
            ct_drag.set_button(gdk::BUTTON_PRIMARY);
            ct_drag.set_propagation_phase(gtk::PropagationPhase::Target);
            ct_drag.connect_drag_begin({
                let this = self.obj().downgrade();
                move |_, x, y| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return,
                    };
                    this.imp()
                        .drag_pos
                        .set(graphene::Point::new(x as f32, y as f32));
                    update_joy_pos(&this, x, y);
                }
            });
            ct_drag.connect_drag_update({
                let this = self.obj().downgrade();
                move |_, x, y| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return,
                    };
                    update_joy_pos(&this, x, y);
                }
            });
            ct_drag.connect_drag_end({
                let this = self.obj().downgrade();
                move |_, x, y| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return,
                    };
                    update_joy_pos(&this, x, y);
                }
            });
            self.obj().add_controller(ct_drag);

            let ct_scroll = gtk::EventControllerScroll::new(
                gtk::EventControllerScrollFlags::BOTH_AXES
                    | gtk::EventControllerScrollFlags::DISCRETE,
            );
            ct_scroll.connect_scroll({
                let this = self.obj().downgrade();
                move |ct, dx, dy| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return glib::Propagation::Proceed,
                    };
                    update_scroll(&this, ct.upcast_ref(), dx, dy);
                    glib::Propagation::Stop
                }
            });
            self.obj().add_controller(ct_scroll);

            self.obj().set_overflow(gtk::Overflow::Hidden);
        }

        fn notify(&self, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "pos-x" | "pos-y" => {
                    self.obj().queue_draw();
                }
                _ => (),
            }
        }
    }
    impl WidgetImpl for Joystick {
        fn measure(&self, _orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            const MIN_SIZE: i32 = 128;
            (MIN_SIZE, for_size.max(MIN_SIZE), -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            const SQRT2_2: f32 = 0.70710678118654752440;

            self.parent_snapshot(snapshot);

            let jx = self.pos_x.get();
            let jy = self.pos_y.get();

            let width = self.obj().width() as f32;
            let height = self.obj().height() as f32;

            let bound_rect = graphene::Rect::new(0.0, 0.0, width, height);
            let center = bound_rect.center();
            let outer = bound_rect.bottom_right();

            let js_pos = js_to_widget(jx, jy, width, height);

            let fg_color = self.obj().color();
            let js_line_color = gdk::RGBA::new(0.0, 0.3, 1.0, 1.0);
            let js_dot_color = gdk::RGBA::RED;

            let outline_width: f32 = 1.0;
            let js_line_width: f32 = 3.0;
            let js_dot_radius: f32 = 4.0;

            // render circle and axes
            {
                let path = {
                    let p = gsk::PathBuilder::new();
                    // axis-aligned ellipse
                    p.move_to(0.0, center.y());
                    p.conic_to(0.0, 0.0, center.x(), 0.0, SQRT2_2);
                    p.conic_to(outer.x(), 0.0, outer.x(), center.y(), SQRT2_2);
                    p.conic_to(outer.x(), outer.y(), center.x(), outer.y(), SQRT2_2);
                    p.conic_to(0.0, outer.y(), 0.0, center.y(), SQRT2_2);

                    p.line_to(outer.x(), center.y());
                    p.move_to(center.x(), 0.0);
                    p.line_to(center.x(), outer.y());

                    p.to_path()
                };
                let stroke = gsk::Stroke::new(outline_width);
                snapshot.append_stroke(&path, &stroke, &fg_color);
            }
            // draw line to JS position
            {
                let path = {
                    let p = gsk::PathBuilder::new();
                    p.move_to(center.x(), center.y());
                    p.line_to(js_pos.x(), js_pos.y());

                    p.to_path()
                };
                let stroke = gsk::Stroke::builder(js_line_width)
                    .line_cap(gsk::LineCap::Round)
                    .build();
                snapshot.append_stroke(&path, &stroke, &js_line_color);
            }
            // draw dot at JS position
            {
                let path = {
                    let p = gsk::PathBuilder::new();
                    p.add_circle(&js_pos, js_dot_radius);

                    p.to_path()
                };
                snapshot.append_fill(&path, gsk::FillRule::EvenOdd, &js_dot_color);
            }
        }
    }

    // MATH UTILITIES

    fn js_to_widget(x: i8, y: i8, w_width: f32, w_height: f32) -> graphene::Point {
        // convert to unsigned value [0, 256) in screen coordinates
        let bx = (x as u8).wrapping_add(128);
        let by = 127u8.wrapping_sub(y as u8);

        // normalize to [0, 1) (this loses no precision since dividing by power of 2)
        let nx = (bx as f32) / 256.0;
        let ny = (by as f32) / 256.0;

        graphene::Point::new(nx * w_width, ny * w_height)
    }

    fn widget_to_js(pos: graphene::Point, w_width: f32, w_height: f32) -> (i8, i8) {
        static UNIT_SQUARE: LazyLock<graphene::Rect> =
            LazyLock::new(|| graphene::Rect::new(0.0, 0.0, 1.0, 1.0));
        static CENTER: LazyLock<graphene::Vec2> = LazyLock::new(|| graphene::Vec2::new(0.5, 0.5));

        let pos = pos.to_vec2();
        let size_vec = graphene::Vec2::new(w_width, w_height);

        let npos = pos.divide(&size_vec);

        let (clip_x, clip_y) = if UNIT_SQUARE.contains_point(&graphene::Point::from_vec2(&npos)) {
            (npos.x(), npos.y())
        } else {
            // direction vector for a ray starting at CENTER and going outwards to npos
            let dir = npos.subtract(&CENTER);

            let (dir_x, dir_y) = (dir.x(), dir.y());
            match (dir_y > dir_x, dir_y > -dir_x) {
                // bottom
                (true, true) => (0.5 + (dir_x / (2.0 * dir_y)), 1.0),
                // left
                (true, false) => (0.0, 0.5 - (dir_y / (2.0 * dir_x))),
                // right
                (false, true) => (1.0, 0.5 + (dir_y / (2.0 * dir_x))),
                // top
                (false, false) => (0.5 - (dir_x / (2.0 * dir_y)), 0.0),
            }
        };

        let jx = ((clip_x * 256.0).round() as u8).wrapping_sub(128) as i8;
        let jy = 127u8.wrapping_sub((clip_y * 256.0).round() as u8) as i8;

        (jx, jy)
    }
}

glib::wrapper! {
    pub struct Joystick(ObjectSubclass<inner::Joystick>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
