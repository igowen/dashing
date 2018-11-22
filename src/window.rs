use gfx_device_gl;
use gfx_window_glutin;
use glutin;
use log::info;
use std;

use crate::graphics::render;
use crate::resources::sprite::SpriteTexture;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;
const GLES_MAJOR_VERSION: u8 = 3;
const GLES_MINOR_VERSION: u8 = 0;

/// `WindowError` represents an error that occurred in the window system.
#[derive(Debug)]
pub enum WindowError {
    /// Generic error.
    GeneralError(String),
    /// Error from the renderer.
    RenderError(render::RenderError),
}

// TODO: Get rid of this, it really sucks.
impl<S> std::convert::From<S> for WindowError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        WindowError::GeneralError(s.to_string())
    }
}

impl std::convert::From<render::RenderError> for WindowError {
    fn from(e: render::RenderError) -> Self {
        WindowError::RenderError(e)
    }
}

/// Helper for constructing windows.
pub struct WindowBuilder<'a> {
    window_title: &'a str,
    width: u32,
    height: u32,
    sprite_texture: &'a SpriteTexture,
    vsync: bool,
    full_screen: bool,
}

impl<'a> WindowBuilder<'a> {
    /// Create a new `WindowBuilder` with the given width and height (measured in sprites, not
    /// pixels).
    pub fn new(
        window_title: &'a str,
        width: u32,
        height: u32,
        sprite_texture: &'a SpriteTexture,
    ) -> Self {
        WindowBuilder {
            window_title: window_title,
            width: width,
            height: height,
            sprite_texture: sprite_texture,
            vsync: true,
            full_screen: false,
        }
    }

    /// Enable/disable vsync.
    pub fn with_vsync(mut self, enable: bool) -> Self {
        self.vsync = enable;

        self
    }

    /// Enable full screen mode.
    pub fn enable_full_screen(mut self) -> Self {
        self.full_screen = true;

        self
    }

    /// Build the window.
    pub fn build(self) -> Result<Window, WindowError> {
        // TODO: Don't create a window bigger than the display.
        let screen_width = (self.width * self.sprite_texture.sprite_width() as u32) as f64;
        let screen_height = (self.height * self.sprite_texture.sprite_height() as u32) as f64;
        info!("Screen dimensions {}x{}", screen_width, screen_height);

        let event_loop = glutin::EventsLoop::new();
        // TODO: Figure out how to deal with hidpi
        let size = glutin::dpi::LogicalSize::from_physical(
            glutin::dpi::PhysicalSize::new(screen_width, screen_height),
            1.0,
        );
        info!("logical size: {:?}", size);
        let window_builder = glutin::WindowBuilder::new()
            .with_title(self.window_title.to_string())
            .with_dimensions(size)
            .with_maximized(self.full_screen)
            .with_decorations(!self.full_screen)
            .with_resizable(false);
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (GL_MAJOR_VERSION, GL_MINOR_VERSION),
                opengles_version: (GLES_MAJOR_VERSION, GLES_MINOR_VERSION),
            })
            .with_gl_profile(glutin::GlProfile::Core)
            .with_vsync(self.vsync)
            .with_double_buffer(Some(true));
        let (window, mut device, mut factory, color_view, depth_view) =
            gfx_window_glutin::init::<render::ColorFormat, render::DepthFormat>(
                window_builder,
                context,
                &event_loop,
            );
        info!(
            "physical size: {:?}",
            size.to_physical(window.get_current_monitor().get_hidpi_factor())
        );

        let command_buffer = factory.create_command_buffer();

        // OpenGL seems to give us an SRGB surface whether we ask for it or not, so we disable it
        // entirely here. This is kind of a hack but it's the only way i've found to get around it.
        unsafe {
            use gl;
            device.with_gl(|gl| {
                gl.Disable(gl::FRAMEBUFFER_SRGB);
            })
        }

        let renderer = render::Renderer::new(
            device,
            factory,
            command_buffer,
            color_view,
            depth_view,
            self.width as usize,
            self.height as usize,
            self.sprite_texture,
        )?;

        Ok(Window {
            event_loop: event_loop,
            window: window,
            renderer: renderer,

            width: self.width,
            height: self.height,
        })
    }
}

/// `Window` is responsible for creating and managing the game window and underlying GL context.
pub struct Window {
    // Handles to device resources we need to hold onto.
    event_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    renderer: render::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory>,

    width: u32,
    height: u32,
}

impl Window {
    /// Render one frame.
    pub(crate) fn render(&mut self) -> Result<(), WindowError> {
        self.renderer.render()?;
        self.window.swap_buffers()?;
        Ok(())
    }

    /// Get a mutable reference to the underlying renderer.
    pub fn renderer_mut(
        &mut self,
    ) -> &mut render::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory> {
        &mut self.renderer
    }

    /// Get an immutable reference to the underlying renderer.
    pub fn renderer(&self) -> &render::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory> {
        &self.renderer
    }

    pub(crate) fn event_loop_mut(&mut self) -> &mut glutin::EventsLoop {
        &mut self.event_loop
    }

    pub(crate) fn event_loop(&self) -> &glutin::EventsLoop {
        &self.event_loop
    }
}
