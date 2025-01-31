use std::{
    ptr::NonNull,
    sync::{Arc, LazyLock, RwLock},
};

use gdk::prelude::*;
use gdk_wayland::ffi::{gdk_wayland_display_get_wl_display, GdkWaylandDisplay};
use glib::translate::*;
use glutin::api::egl::display::Display as EGLDisplay;
use m64prs_gtk_utils::quark;
use raw_window_handle::WaylandDisplayHandle;
use wayland_backend::client::Backend;
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::WlCompositor,
        wl_display::WlDisplay,
        wl_region::WlRegion,
        wl_registry::WlRegistry,
        wl_subcompositor::WlSubcompositor,
        wl_subsurface::WlSubsurface,
        wl_surface::{self, WlSurface},
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};

use super::macros::empty_dispatch;

pub struct DisplayState {
    pub connection: Connection,
    pub display: WlDisplay,

    pub queue: RwLock<Queue>,

    pub compositor: WlCompositor,
    pub subcompositor: WlSubcompositor,

    pub egl_display: EGLDisplay,
}

pub struct Queue {
    pub state: QueueState,
    pub events: EventQueue<QueueState>,
}

impl Queue {
    pub fn handle(&self) -> QueueHandle<QueueState> {
        self.events.handle()
    }
    pub fn roundtrip(&mut self) {
        self.events
            .roundtrip(&mut self.state)
            .expect("roundtrip should not fail");
    }
}

#[derive(Default)]
pub struct QueueState {}

empty_dispatch!(impl Dispatch<WlRegistry, GlobalListContents> for QueueState);
empty_dispatch!(impl Dispatch<WlCompositor, ()> for QueueState);
empty_dispatch!(impl Dispatch<WlSubcompositor, ()> for QueueState);
empty_dispatch!(impl Dispatch<WlRegion, ()> for QueueState);
empty_dispatch!(impl Dispatch<WlSubsurface, ()> for QueueState);

impl Dispatch<WlSurface, ()> for QueueState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        event: <WlSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        #[allow(unused)]
        match event {
            wl_surface::Event::Enter { output } => {}
            wl_surface::Event::Leave { output } => {}
            wl_surface::Event::PreferredBufferScale { factor } => {}
            wl_surface::Event::PreferredBufferTransform { transform } => {}
            _ => (),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait WaylandDisplayExt: sealed::Sealed {
    fn display_state(&self) -> Arc<DisplayState>;
}

impl sealed::Sealed for gdk_wayland::WaylandDisplay {}
impl WaylandDisplayExt for gdk_wayland::WaylandDisplay {
    fn display_state(&self) -> Arc<DisplayState> {
        static M64PRS_WAYLAND_DISPLAY_STATE: LazyLock<glib::Quark> =
            quark!("io.github.jgcodes2020.m64prs.wayland_display_state");

        // if the display already has globals set, then return then
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            if let Some(p_data) = self.qdata::<Arc<DisplayState>>(*M64PRS_WAYLAND_DISPLAY_STATE) {
                return Arc::clone(p_data.as_ref());
            }
        }

        // acquire FFI objects (wl_display) from GDK
        let (connection, display) = unsafe {
            let ffi_display: Stash<'_, *mut GdkWaylandDisplay, _> = self.to_glib_none();
            let display_ptr = gdk_wayland_display_get_wl_display(ffi_display.0);

            let display_backend = Backend::from_foreign_display(display_ptr as *mut _);
            let conn = Connection::from_backend(display_backend);
            let display = conn.display();

            (conn, display)
        };

        // create new event queue for our own stuff
        let (globals, event_queue) = registry_queue_init::<QueueState>(&connection)
            .expect("Failed to acquire Wayland globals");
        let queue_state = QueueState::default();
        let qh = event_queue.handle();

        // bind compositor/subcompositor globals
        let compositor: WlCompositor = globals
            .bind(&qh, 5..=6, ())
            .expect("expected support for wl_compositor@v5 or @v6");
        let subcompositor: WlSubcompositor = globals
            .bind(&qh, 1..=1, ())
            .expect("expected support for wl_subcompositor@v1");

        // Bind EGL display (GTK has a function for this but it isn't exposed)
        let egl_display = unsafe {
            EGLDisplay::new(
                WaylandDisplayHandle::new(NonNull::new(display.id().as_ptr() as *mut _).unwrap())
                    .into(),
            )
            .expect("EGL display creation should succeed")
        };

        let state = Arc::new(DisplayState {
            connection,
            display,
            queue: RwLock::new(Queue {
                state: queue_state,
                events: event_queue,
            }),
            compositor,
            subcompositor,
            egl_display,
        });

        // set the state now
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            self.set_qdata(*M64PRS_WAYLAND_DISPLAY_STATE, state);
            Arc::clone(
                self.qdata::<Arc<DisplayState>>(*M64PRS_WAYLAND_DISPLAY_STATE)
                    .unwrap()
                    .as_ref(),
            )
        }
    }
}
