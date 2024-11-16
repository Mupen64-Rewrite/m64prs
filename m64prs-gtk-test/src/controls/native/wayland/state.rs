use std::
    sync::{Arc, LazyLock, RwLock}
;

use gdk::prelude::*;
use gdk_wayland::ffi::{
    gdk_wayland_display_get_wl_display, gdk_wayland_surface_get_wl_surface, GdkWaylandDisplay,
    GdkWaylandSurface,
};
use glib::translate::*;
use wayland_backend::client::{Backend, ObjectId};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::WlCompositor, wl_display::WlDisplay, wl_region::WlRegion, wl_registry::{WlRegistry}, wl_subcompositor::WlSubcompositor, wl_subsurface::WlSubsurface, wl_surface::{self, WlSurface}
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
        self.events.roundtrip(&mut self.state)
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
            wl_surface::Event::Enter { output } => {
                
            },
            wl_surface::Event::Leave { output } => {
                
            },
            wl_surface::Event::PreferredBufferScale { factor } => {
                
            },
            wl_surface::Event::PreferredBufferTransform { transform } => {
                
            },
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
pub trait WaylandSurfaceExt: sealed::Sealed {
    fn wl_surface(&self) -> WlSurface;
}

impl sealed::Sealed for gdk_wayland::WaylandDisplay {}
impl WaylandDisplayExt for gdk_wayland::WaylandDisplay {
    fn display_state(&self) -> Arc<DisplayState> {
        static M64PRS_WAYLAND_DISPLAY_STATE: LazyLock<glib::Quark> = LazyLock::new(|| {
            const QUARK_NAME: &glib::GStr =
                glib::gstr!("io.github.jgcodes2020.m64prs.wayland_display_state");
            glib::Quark::from_static_str(QUARK_NAME)
        });

        // if the display already has globals set, then return then
        unsafe {
            // SAFETY: this key is always used with Arc<DisplayState>.
            if let Some(p_data) = self.qdata::<Arc<DisplayState>>(*M64PRS_WAYLAND_DISPLAY_STATE) {
                return Arc::clone(p_data.as_ref());
            }
        }

        // acquire wl_display from GDK
        let (connection, display) = unsafe {
            let ffi_display: Stash<'_, *mut GdkWaylandDisplay, _> = self.to_glib_none();
            let display_ptr = gdk_wayland_display_get_wl_display(ffi_display.0);

            let display_backend = Backend::from_foreign_display(display_ptr as *mut _);
            let conn = Connection::from_backend(display_backend);
            let display = conn.display();
            (conn, display)
        };

        let (globals, event_queue) = registry_queue_init::<QueueState>(&connection)
            .expect("Failed to acquire Wayland globals");

        let queue_state = QueueState::default();

        let qh = event_queue.handle();

        let compositor: WlCompositor = globals
            .bind(&qh, 6..=6, ())
            .expect("expected support for wl_compositor@v6");
        let subcompositor: WlSubcompositor = globals
            .bind(&qh, 1..=1, ())
            .expect("expected support for wl_subcompositor@v1");

        let state = Arc::new(DisplayState {
            connection,
            display,
            queue: RwLock::new(Queue {
                state: queue_state,
                events: event_queue,
            }),
            compositor,
            subcompositor,
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

impl sealed::Sealed for gdk_wayland::WaylandSurface {}
impl WaylandSurfaceExt for gdk_wayland::WaylandSurface {
    fn wl_surface(&self) -> WlSurface {
        static M64PRS_WAYLAND_SURFACE: LazyLock<glib::Quark> = LazyLock::new(|| {
            const QUARK_NAME: &glib::GStr =
                glib::gstr!("io.github.jgcodes2020.m64prs.wayland_surface");
            glib::Quark::from_static_str(QUARK_NAME)
        });

        unsafe {
            // SAFETY: this key is always WlSurface.
            if let Some(p_data) = self.qdata::<WlSurface>(*M64PRS_WAYLAND_SURFACE) {
                return p_data.as_ref().clone();
            }
        }

        let gdk_display = self
            .display()
            .downcast::<gdk_wayland::WaylandDisplay>()
            .unwrap();
        let st = gdk_display.display_state();
        
        // acquire wl_surface from GDK
        let surface = unsafe {
            let ffi_surface: Stash<'_, *mut GdkWaylandSurface, _> = self.to_glib_none();
            let surface_ptr = gdk_wayland_surface_get_wl_surface(ffi_surface.0);

            let id = ObjectId::from_ptr(WlSurface::interface(), surface_ptr as *mut _)
                .expect("gdk_wayland::WaylandSurface::wl_surface should return a valid wl_surface");
            let surface = WlSurface::from_id(&st.connection, id);
            surface
        };

        unsafe {
            // SAFETY: this key is always WlSurface.
            self.set_qdata(*M64PRS_WAYLAND_SURFACE, surface);
            self.qdata::<WlSurface>(*M64PRS_WAYLAND_SURFACE).unwrap().as_ref().clone()
        }
    }
}
