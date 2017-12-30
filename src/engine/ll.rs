use gfx_device_gl;
use gfx_window_sdl;
use sdl2;
use std;

use engine::mid;
use engine::renderer;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

/// LLEngine provides the lowest level abstraction on the graphics subsystem. You hopefully won't
/// need to interact with it directly, but most of the functionality is public just in case.
// TODO(igowen): should this be generic over resource types?
pub struct LLEngine {
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

/// LLEngineError represents an error that occurred in the low-level engine.
#[derive(Debug)]
pub enum LLEngineError {
    /// Generic error.
    GeneralError(String),
    /// Error from the SDL subsystem.
    SDLError(String),
    /// Error from the renderer.
    RenderError(renderer::RenderError),
}

impl<S> std::convert::From<S> for LLEngineError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        LLEngineError::GeneralError(s.to_string())
    }
}

impl std::convert::From<renderer::RenderError> for LLEngineError {
    fn from(e: renderer::RenderError) -> Self {
        LLEngineError::RenderError(e)
    }
}

impl LLEngine {
    /// Create a new `LLEngine` with the given width and height (measured in characters, not
    /// pixels).
    pub fn new(width: u32, height: u32) -> Result<Self, LLEngineError> {
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
        let builder = video.window("rlb", screen_width * 2, screen_height * 2);
        let window_result =
            gfx_window_sdl::init::<renderer::ColorFormat, renderer::DepthFormat>(&video, builder);
        let (window, gl_context, device, mut factory, color_view, depth_view);
        match window_result {
            Err(e) => {
                return Err(LLEngineError::SDLError(format!("SDL init error: {:?}", e)));
            }
            Ok((w, c, d, f, cv, dv)) => {
                // Make sure we hold on to all of these -- if the GL context gets dropped, we can't do any GL
                // operations, even though we don't interact with it directly.
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

        Ok(LLEngine {
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
    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, LLEngineError> {
        self.renderer.render()?;
        self.window.gl_swap_window();
        Ok(self.event_pump.poll_iter())
    }

    /// Get the current frames per second. This is based on a rolling average, not the
    /// instantaneous measurement.
    pub fn get_fps(&self) -> f32 {
        self.renderer.get_fps()
    }

    /// Get the number of frames that have been rendered.
    pub fn get_frame_counter(&self) -> u32 {
        self.renderer.get_frame_counter()
    }
}

impl<'a> LLEngine {
    /// Update the character matrix with the provided data.
    pub fn update<T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a mid::CharCell>,
    {
        self.renderer.update(data);
    }
}
