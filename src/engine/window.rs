use gfx_device_gl;
use gfx_window_sdl;
use sdl2;
use std;

use engine::renderer;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

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

impl Window {
    /// Create a new `Window` with the given width and height (measured in characters, not
    /// pixels).
    pub fn new(window_title: &str, width: u32, height: u32) -> Result<Self, WindowError> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        {
            let gl = video.gl_attr();
            gl.set_context_profile(sdl2::video::GLProfile::Core);
            gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        }

        let screen_width = width * renderer::FONT_WIDTH as u32;
        let screen_height = height * renderer::FONT_HEIGHT as u32;

        // TODO: HiDPI check for the x2 factor here.
        // TODO: Don't create a window bigger than the display.
        let builder = video.window(window_title, screen_width * 2, screen_height * 2);
        let window_result =
            gfx_window_sdl::init::<renderer::ColorFormat, renderer::DepthFormat>(&video, builder);
        let (window, gl_context, device, mut factory, color_view, depth_view);
        match window_result {
            Err(e) => {
                return Err(WindowError::SDLError(format!("SDL init error: {:?}", e)));
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

        // Disable vsync.
        //video.gl_set_swap_interval(0);

        let event_pump = sdl_context.event_pump()?;

        let command_buffer = factory.create_command_buffer();

        let renderer = renderer::Renderer::new(
            device,
            factory,
            command_buffer,
            color_view,
            depth_view,
            width as usize,
            height as usize,
        )?;

        Ok(Window {
            sdl_context: sdl_context,
            event_pump: event_pump,
            video: video,
            window: window,
            gl_context: gl_context,
            renderer: renderer,

            width: width,
            height: height,
        })
    }

    /// Render one frame, and return an iterator over the events that have elapsed since the last
    /// frame.
    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, WindowError> {
        self.renderer.render()?;
        self.window.gl_swap_window();
        Ok(self.event_pump.poll_iter())
    }
}
