use gfx_device_gl;
use gfx_window_sdl;
use sdl2;
use std;

use engine::renderer;
use resources::sprite::SpriteTexture;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

/// `WindowError` represents an error that occurred in the window system.
#[derive(Debug)]
pub enum WindowError {
    /// Generic error.
    GeneralError(String),
    /// Error from the SDL subsystem.
    SDLError(String),
    /// Error from the renderer.
    RenderError(renderer::RenderError),
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

impl std::convert::From<renderer::RenderError> for WindowError {
    fn from(e: renderer::RenderError) -> Self {
        WindowError::RenderError(e)
    }
}

/// Helper for constructing windows.
pub struct WindowBuilder<'a> {
    window_title: &'a str,
    width: u32,
    height: u32,
    sprite_texture: &'a SpriteTexture,
    enable_vsync: bool,
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
            enable_vsync: true,
            full_screen: false,
        }
    }

    /// Disable vsync.
    pub fn disable_vsync(&'a mut self) -> &'a mut Self {
        self.enable_vsync = false;

        self
    }

    /// Enable full screen mode.
    pub fn enable_full_screen(&'a mut self) -> &'a mut Self {
        self.full_screen = true;

        self
    }

    /// Build the window.
    pub fn build(&self) -> Result<Window, WindowError> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        {
            let gl = video.gl_attr();
            gl.set_context_profile(sdl2::video::GLProfile::Core);
            gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        }

        let screen_width = self.width * self.sprite_texture.sprite_width() as u32;
        let screen_height = self.height * self.sprite_texture.sprite_height() as u32;

        // TODO: HiDPI check for the x2 factor here.
        // TODO: Don't create a window bigger than the display.
        let mut builder = video.window(self.window_title, screen_width, screen_height);
        if self.full_screen {
            builder.fullscreen_desktop();
        }
        let window_result =
            gfx_window_sdl::init::<renderer::ColorFormat, renderer::DepthFormat>(&video, builder);
        let (window, gl_context, device, mut factory, color_view, depth_view);
        match window_result {
            Err(e) => {
                return Err(WindowError::SDLError(
                    format!("Couldn't initialize SDL: {:?}", e),
                ));
            }
            Ok((w, c, d, f, cv, dv)) => {
                // Make sure we hold on to all of these -- if the GL context gets dropped, we can't
                // do any GL operations, even though we don't interact with it directly.
                window = w;
                gl_context = c;
                device = d;
                factory = f;
                color_view = cv;
                depth_view = dv;
            }
        };

        if self.enable_vsync {
            video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
        } else {
            video.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate);
        }

        let event_pump = sdl_context.event_pump()?;

        let command_buffer = factory.create_command_buffer();

        let renderer = renderer::Renderer::new(
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
            sdl_context: sdl_context,
            event_pump: event_pump,
            video: video,
            window: window,
            gl_context: gl_context,
            renderer: renderer,

            width: self.width,
            height: self.height,
        })
    }
}

/// `Window` is responsible for creating and managing the game window and underlying GL context.
pub struct Window {
    // Handles to device resources we need to hold onto.
    sdl_context: sdl2::Sdl,
    event_pump: sdl2::EventPump,
    video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    renderer: renderer::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory>,

    width: u32,
    height: u32,
}

impl Window {
    /// Render one frame, and return an iterator over the events that have elapsed since the last
    /// frame.
    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, WindowError> {
        self.renderer.render()?;
        self.window.gl_swap_window();
        Ok(self.event_pump.poll_iter())
    }

    /// Get a mutable reference to the underlying renderer.
    pub fn renderer_mut(
        &mut self,
    ) -> &mut renderer::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory> {
        &mut self.renderer
    }

    /// Get an immutable reference to the underlying renderer.
    pub fn renderer(&self) -> &renderer::Renderer<gfx_device_gl::Device, gfx_device_gl::Factory> {
        &self.renderer
    }
}
