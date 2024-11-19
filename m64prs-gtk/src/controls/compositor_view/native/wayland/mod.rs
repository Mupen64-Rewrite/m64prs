use std::{
    ffi::{c_void, CString},
    num::NonZero,
    ptr::NonNull,
    sync::Arc,
};

use egl::{EGLContextExt, EGLImage};
use gdk::prelude::*;
use gdk_wayland::prelude::WaylandSurfaceExtManual;
use glutin::{
    api::egl::{
        context::PossiblyCurrentContext as EGLPossiblyCurrentContext,
        surface::Surface as EGLSurface,
    },
    config::{ColorBufferType, ConfigSurfaceTypes, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, Version},
    display::DisplayApiPreference,
    prelude::*,
    surface::{PbufferSurface, SurfaceAttributesBuilder},
};
use raw_window_handle::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
    WindowHandle,
};
use slotmap::DenseSlotMap;
use state::{DisplayState, WaylandDisplayExt};
use wayland_client::{
    protocol::{wl_display::WlDisplay, wl_subsurface::WlSubsurface, wl_surface::WlSurface},
    Proxy,
};

use crate::utils::gl::{self, types::GLuint};

use super::{NativeCompositor, NativeView, NativeViewKey};

mod egl;
mod macros;
mod state;

pub struct WaylandCompositor {
    display_state: Arc<DisplayState>,
    views: DenseSlotMap<NativeViewKey, WaylandView>,

    current_bounds: dpi::PhysicalSize<u32>,
    mapped: bool,

    parent_surface: WlSurface,
    surface: WlSurface,
    subsurface: WlSubsurface,

    egl_context: EGLPossiblyCurrentContext,
    egl_surface: EGLSurface<PbufferSurface>,
    egl_image: EGLImage,

    gl: gl::Gl,
    fbo: GLuint,
    rbo: GLuint,
}

struct WaylandView {
    display_state: Arc<DisplayState>,

    surface: WlSurface,
    subsurface: WlSubsurface,

    transparent: bool,
    position: dpi::PhysicalPosition<i32>,
    size: dpi::PhysicalSize<u32>,
}

struct WaylandViewHandle {
    view_key: NativeViewKey,
    display: WlDisplay,
    surface: WlSurface,
}

impl WaylandCompositor {
    pub fn new(
        gdk_surface: &gdk_wayland::WaylandSurface,
        position: dpi::PhysicalPosition<i32>,
    ) -> Self {
        let gdk_display = gdk_surface
            .display()
            .downcast::<gdk_wayland::WaylandDisplay>()
            .unwrap();
        let st = gdk_display.display_state();

        let mut queue = st.queue.write().unwrap();
        let qh = queue.handle();

        let surface = st.compositor.create_surface(&qh, ());
        let parent_surface = gdk_surface
            .wl_surface()
            .expect("Parent should have Wayland surface");

        // Create the root subsurface
        let subsurface = st
            .subcompositor
            .get_subsurface(&surface, &parent_surface, &qh, ());
        subsurface.set_desync();
        subsurface.set_position(position.x, position.y);

        // make the subsurface click-through
        {
            let input_region = st.compositor.create_region(&qh, ());
            surface.set_input_region(Some(&input_region));
        }

        // ensure the subsurface's position is updated
        // and unlock the queue
        parent_surface.commit();
        queue.roundtrip();
        drop(queue);

        // setup EGL using Glutin's API
        let egl_display = &st.egl_display;

        let egl_config = {
            let builder = ConfigTemplateBuilder::new()
                .with_buffer_type(ColorBufferType::Rgb {
                    r_size: 8,
                    g_size: 8,
                    b_size: 8,
                })
                .with_alpha_size(0);

            unsafe { egl_display.find_configs(builder.build()) }
                .expect("there shouldn't be problems generating the config iterator")
                .find(|config| {
                    config
                        .config_surface_types()
                        .contains(ConfigSurfaceTypes::PBUFFER)
                })
                .expect("there should be a config supporting PBuffers")
        };

        // Create offscreen surface since we're creating a
        // separate buffer later on
        let egl_surface = {
            let builder = SurfaceAttributesBuilder::<PbufferSurface>::new()
                .build(NonZero::new(1).unwrap(), NonZero::new(1).unwrap());

            unsafe { egl_display.create_pbuffer_surface(&egl_config, &builder) }
                .expect("PBuffer creation should succeed")
        };

        let egl_context = {
            let builder = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version { major: 4, minor: 5 })))
                .with_profile(GlProfile::Compatibility);

            let context = unsafe { egl_display.create_context(&egl_config, &builder.build(None)) }
                .expect("context creation should succeed");
            context
                .make_current_surfaceless()
                .expect("context should be able to be made current")
        };

        let gl = gl::Gl::load_with(|s| egl_display.get_proc_address(&CString::new(s).unwrap()));

        // OpenGL: setup an FBO and colour RBO and connect it to an EGL buffer
        let (rbo, fbo, egl_image) = unsafe {
            // for now, set
            let mut rbo: u32 = 0;
            gl.CreateRenderbuffers(1, &mut rbo);
            debug_assert!(rbo != 0);
            gl.NamedRenderbufferStorage(rbo, gl::RGBA8, 1, 1);

            let mut fbo: u32 = 0;
            gl.CreateFramebuffers(1, &mut fbo);
            debug_assert!(rbo != 0);
            gl.NamedFramebufferRenderbuffer(fbo, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, rbo);

            let image = egl_context
                .create_image_renderbuffer(rbo)
                .expect("Creating EGLImage should succeed");
            (rbo, fbo, image)
        };

        Self {
            // basic display state
            display_state: st,
            views: DenseSlotMap::with_key(),
            // bounds tracking
            current_bounds: dpi::PhysicalSize::new(0, 0),
            mapped: false,
            // Wayland
            parent_surface,
            surface,
            subsurface,
            // EGL
            egl_context,
            egl_surface,
            egl_image,
            // OpenGL
            gl,
            fbo,
            rbo,
        }
    }

    fn compute_bounds(&self) -> dpi::PhysicalSize<u32> {
        let (max_w, max_h) = self
            .views
            .iter()
            .fold((0u32, 0u32), |(max_w, max_h), (_, view)| {
                let max_w = u32::max(max_w, (view.position.x + view.size.width as i32) as u32);
                let max_h = u32::max(max_h, (view.position.y + view.size.height as i32) as u32);
                (max_w, max_h)
            });
        dpi::PhysicalSize::new(max_w, max_h)
    }

    fn on_bounds_changed(&mut self) {
        self.egl_context
            .make_current_surfaceless()
            .expect("context being made current should succeed");

        let st = &*self.display_state;
        let queue = st.queue.write().unwrap();
        let qh = queue.handle();

        let size = &self.current_bounds;
        let gl = &self.gl;

        // Set the opaque region of the surface
        {
            let opaque_region = st.compositor.create_region(&qh, ());
            opaque_region.add(0, 0, size.width as i32, size.height as i32);
            self.surface.set_opaque_region(Some(&opaque_region));
        }
        drop(queue);

        // resize and repaint the buffer
        unsafe {
            gl.NamedRenderbufferStorage(self.rbo, gl::RGBA8, size.width as i32, size.height as i32);

            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.Finish();
        }

        // remap the buffer if necessary
        self.set_mapped(self.mapped);
    }
}

impl NativeCompositor for WaylandCompositor {
    fn new_view(&mut self, attrs: super::NativeViewAttributes) -> Box<dyn super::NativeView> {
        let st = &*self.display_state;

        let mut queue = st.queue.write().unwrap();
        let qh = queue.handle();

        let size: dpi::PhysicalSize<u32> = attrs.surface_size;
        let position: dpi::PhysicalPosition<i32> = attrs.position;
        let transparent: bool = attrs.transparent;

        let surface = st.compositor.create_surface(&qh, ());
        let parent_surface = self.surface.clone();

        let subsurface = st
            .subcompositor
            .get_subsurface(&surface, &parent_surface, &qh, ());
        subsurface.set_desync();
        subsurface.set_position(position.x, position.y);

        {
            let input_region = st.compositor.create_region(&qh, ());
            input_region.subtract(0, 0, size.width as i32, size.height as i32);
            surface.set_input_region(Some(&input_region));

            if !transparent {
                let opaque_region = st.compositor.create_region(&qh, ());
                opaque_region.add(0, 0, size.width as i32, size.height as i32);
                surface.set_opaque_region(Some(&opaque_region));
            }
        }

        parent_surface.commit();
        queue.roundtrip();
        drop(queue);

        let view = WaylandView {
            display_state: self.display_state.clone(),
            surface: surface.clone(),
            subsurface,
            transparent,
            position,
            size,
        };

        let view_key = self.views.insert(view);
        let display = st.display.clone();

        // recompute bounds
        self.current_bounds = self.compute_bounds();
        self.on_bounds_changed();

        Box::new(WaylandViewHandle {
            view_key,
            display,
            surface,
        })
    }

    fn delete_view(&mut self, view_key: NativeViewKey) {
        if self.views.remove(view_key).is_none() {
            panic!("delete_view should be called with a valid key")
        };

        // recompute bounds
        self.current_bounds = self.compute_bounds();
        self.on_bounds_changed();
    }

    fn set_view_bounds(
        &mut self,
        view_key: NativeViewKey,
        position: Option<dpi::PhysicalPosition<i32>>,
        size: Option<dpi::PhysicalSize<u32>>,
    ) {

        let view = self.views.get_mut(view_key)
            .expect("set_view_bounds requires a valid key");

        if position.is_none() && size.is_none() {
            return;
        }

        if let Some(position) = position {
            view.set_position(position);
        }
        if let Some(size) = size {
            view.set_size(size);
        }
        self.surface.commit();

        // recompute bounds
        self.current_bounds = self.compute_bounds();
        self.on_bounds_changed();
    }

    fn restack_view(&mut self, view_key: NativeViewKey, stack_order: super::StackOrder) {
        let view = self.views.get(view_key)
            .expect("set_view_bounds requires a valid key");
        match stack_order {
            super::StackOrder::StackAbove(ref_view_key) => {
                let ref_view = self.views.get(view_key)
                    .expect("set_view_bounds requires a valid key");
                
                view.subsurface.place_above(&ref_view.surface);
            },
            super::StackOrder::StackBelow(ref_view_key) => {
                let ref_view = self.views.get(view_key)
                    .expect("set_view_bounds requires a valid key");
                
                view.subsurface.place_below(&ref_view.surface);
            },
        }
    }

    fn total_bounds(&self) -> dpi::PhysicalSize<u32> {
        self.current_bounds
    }

    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        self.subsurface.set_position(position.x, position.y);
        self.parent_surface.commit();
    }

    fn set_mapped(&mut self, mapped: bool) {
        let buffer = mapped.then(|| unsafe {
            self.egl_image
                .get_wayland_buffer(&self.display_state.connection)
        });
        self.surface.attach(buffer.as_ref(), 0, 0);
        self.surface.commit();

        self.mapped = mapped;
    }
}

impl WaylandView {
    fn set_position(&mut self, position: dpi::PhysicalPosition<i32>) {
        self.position = position;
        self.subsurface.set_position(position.x, position.y);
    }

    fn set_size(&mut self, size: dpi::PhysicalSize<u32>) {
        self.size = size;
    }
}

impl NativeView for WaylandViewHandle {
    fn key(&self) -> NativeViewKey {
        self.view_key
    }

    fn display_handle_src(&self) -> &dyn raw_window_handle::HasDisplayHandle {
        self
    }

    fn window_handle_src(&self) -> &dyn raw_window_handle::HasWindowHandle {
        self
    }

    fn gl_api_preference(&self) -> DisplayApiPreference {
        DisplayApiPreference::Egl
    }
}
impl HasDisplayHandle for WaylandViewHandle {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let raw_handle = WaylandDisplayHandle::new(
            NonNull::new(self.display.id().as_ptr() as *mut c_void).unwrap(),
        );
        Ok(unsafe { DisplayHandle::borrow_raw(raw_handle.into()) })
    }
}
impl HasWindowHandle for WaylandViewHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let raw_handle = WaylandWindowHandle::new(
            NonNull::new(self.surface.id().as_ptr() as *mut c_void).unwrap(),
        );
        Ok(unsafe { WindowHandle::borrow_raw(raw_handle.into()) })
    }
}
