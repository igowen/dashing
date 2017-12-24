use gfx;
use gfx_core;
use gfx_device_gl;
use gfx_window_sdl;
use image;
use sdl2;
use std;
use time;

use gfx::Device;
use gfx::Factory;
use gfx::traits::FactoryExt;

use ::engine::mid;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

const FONT_WIDTH: u32 = 12;
const FONT_HEIGHT: u32 = 12;

type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    vertex Instance {
        translate: [f32; 2] = "a_Translate",
        color: [f32; 4] = "a_FgColor",
        bg_color: [f32; 4] = "a_BgColor",
        character: u32 = "a_Character",
    }

    constant Locals {
        dim: [f32; 2] = "u_ScreenCharDim",
        font_dim: [f32; 2] = "u_FontCharDim",
    }

    constant ScreenLocals {
        screen_dimensions: [f32; 2] = "u_ScreenDimensions",
        frame_counter: u32 = "u_FrameCounter",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        instance: gfx::InstanceBuffer<Instance> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        screen_target: gfx::RenderTarget<ColorFormat> = "IntermediateTarget",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
    }

    pipeline screen_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        screen_tex: gfx::TextureSampler<[f32; 4]> = "t_ScreenTexture",
        locals: gfx::ConstantBuffer<ScreenLocals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
            translate: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.0, 0.0, 0.0, 1.0],
            character: 0,
        }
    }
}

// Vertices for character cell quads.
const QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
];

// Vertices for the screen quad. Only difference here is the UV coordinates, which we could
// probably handle in the shader but 4 redundant vertices isn't the end of the world.
const SCREEN_QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [1.0, -1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        pos: [-1.0, -1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        pos: [-1.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
];

// Triangulation for the above vertices, shared by both the cell quads and the screen quad.
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

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
    device: gfx_window_sdl::Device,
    factory: gfx_window_sdl::Factory,
    color_view: gfx_core::handle::RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
    depth_view: gfx_core::handle::DepthStencilView<gfx_device_gl::Resources, DepthFormat>,
    pipeline: gfx::pso::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    screen_pipeline: gfx::pso::PipelineState<gfx_device_gl::Resources, screen_pipe::Meta>,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,

    // GPU-side resources.
    vertex_slice: gfx::Slice<gfx_device_gl::Resources>,
    screen_vertex_slice: gfx::Slice<gfx_device_gl::Resources>,
    upload_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Instance>,
    pipeline_data: pipe::Data<gfx_device_gl::Resources>,
    screen_pipeline_data: screen_pipe::Data<gfx_device_gl::Resources>,

    // CPU-side resources.
    width: u32,
    height: u32,
    instance_count: u32,
    instances: Box<[Instance]>,
    frame_counter: u32,

    // Engine metadata.
    fps: f64,
    last_render_time_ns: u64,
}

/// LLEngineError represents an error that occurred in the low-level engine.
#[derive(Debug)]
pub enum LLEngineError {
    /// Generic error.
    GeneralError(String),
    /// Error from the SDL subsystem.
    SDLError(String),
    /// Error from the OpenGL subsystem.
    OpenGLError(String),
}

impl<S> std::convert::From<S> for LLEngineError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        LLEngineError::GeneralError(s.to_string())
    }
}

// TODO: move this to mid.
impl<'a> From<&'a mid::CharCell> for Instance {
    fn from(c: &'a mid::CharCell) -> Self {
        Instance {
            character: c.get_character(),
            color: c.get_fg_color(),
            bg_color: c.get_bg_color(),
            translate: [0.0, 0.0],
        }
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

        let screen_width = width * FONT_WIDTH;
        let screen_height = height * FONT_HEIGHT;

        // TODO: HiDPI check for the x2 factor here.
        let builder = video.window("rlb", screen_width * 2, screen_height * 2);
        let window_result = gfx_window_sdl::init::<ColorFormat, DepthFormat>(builder);
        let (window, gl_context, device, mut factory, color_view, depth_view);
        match window_result {
            Err(e) => {
                return Err(LLEngineError::SDLError(format!("SDL init error: {:?}", e)));
            }
            Ok(v) => {
                // Make sure we hold on to all of these -- if the GL context gets dropped, we can't do any GL
                // operations, even though we don't interact with it directly.
                window = v.0;
                gl_context = v.1;
                device = v.2;
                factory = v.3;
                color_view = v.4;
                depth_view = v.5;
            }
        };

        // Disable vsync.
        //video.gl_set_swap_interval(0);

        let pso: gfx::pso::PipelineState<gfx_device_gl::Resources, pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("shader/cell.glslv"),
                include_bytes!("shader/cell.glslf"),
                pipe::new(),
            )?;

        let screen_pso: gfx::pso::PipelineState<
            gfx_device_gl::Resources,
            screen_pipe::Meta,
        > = factory.create_pipeline_simple(
            include_bytes!("shader/screen.glslv"),
            include_bytes!("shader/screen.glslf"),
            screen_pipe::new(),
        )?;

        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
        let (vertex_buffer, mut slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);
        let (screen_vertex_buffer, screen_slice) =
            factory.create_vertex_buffer_with_slice(&SCREEN_QUAD_VERTICES, &QUAD_INDICES[..]);
        let instance_count = width * height;

        slice.instances = Some((instance_count, 0));

        let locals = Locals {
            dim: [width as f32, height as f32],
            font_dim: [16.0, 16.0],
        };

        let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let instance_buffer = factory.create_buffer(
            instance_count as usize,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Data,
            gfx::TRANSFER_DST,
        )?;

        let mut instance_templates = vec![Instance::default(); (width * height) as usize];
        for x in 0..width {
            for y in 0..height {
                instance_templates[(y * width + x) as usize] = Instance {
                    translate: [
                        -1.0 + (x as f32 * 2.0 / width as f32),
                        1.0 - ((y as f32 + 1.0) * 2.0 / height as f32),
                    ],
                    color: [1.0, 1.0, 1.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 0,
                }
            }
        }

        let upload = factory.create_upload_buffer::<Instance>(
            instance_count as usize,
        )?;

        let locals_buffer = factory.create_buffer_immutable(
            &[locals],
            gfx::buffer::Role::Constant,
            gfx::Bind::empty(),
        )?;

        let screen_locals_buffer = factory.create_constant_buffer(1);

        let (_, screen_texture, render_target) = factory.create_render_target(
            screen_width as u16,
            screen_height as u16,
        )?;

        let screen_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let texture = gfx_load_texture(&mut factory);

        let intermediate_data = pipe::Data {
            vbuf: vertex_buffer,
            instance: instance_buffer,
            tex: (texture, sampler),
            screen_target: render_target,
            locals: locals_buffer,
        };

        let final_data = screen_pipe::Data {
            vbuf: screen_vertex_buffer,
            screen_tex: (screen_texture, screen_sampler),
            out: color_view.clone(),
            locals: screen_locals_buffer,
        };

        let event_pump = sdl_context.event_pump()?;

        Ok(LLEngine {
            sdl_context: sdl_context,
            event_pump: event_pump,
            video: video,
            window: window,
            gl_context: gl_context,
            device: device,
            factory: factory,
            color_view: color_view,
            depth_view: depth_view,
            pipeline: pso,
            screen_pipeline: screen_pso,
            encoder: encoder,

            vertex_slice: slice,
            screen_vertex_slice: screen_slice,
            upload_buffer: upload,
            pipeline_data: intermediate_data,
            screen_pipeline_data: final_data,

            width: width,
            height: height,
            instance_count: instance_count,
            instances: instance_templates.into_boxed_slice(),
            frame_counter: 0,
            fps: 0.0,
            last_render_time_ns: 0,
        })
    }

    /// Render one frame, and return an iterator over the events that have elapsed since the last
    /// frame.
    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, LLEngineError> {
        {
            let mut writer = self.factory.write_mapping(&self.upload_buffer)?;
            writer.copy_from_slice(&self.instances[..]);
        }

        self.encoder.clear(
            &self.pipeline_data.screen_target,
            [0.2, 0.0, 0.0, 1.0],
        );

        self.encoder.copy_buffer(
            &self.upload_buffer,
            &self.pipeline_data.instance,
            0,
            0,
            self.upload_buffer.len(),
        )?;

        self.encoder.draw(
            &self.vertex_slice,
            &self.pipeline,
            &self.pipeline_data,
        );

        self.encoder.update_constant_buffer(
            &self.screen_pipeline_data.locals,
            &ScreenLocals {
                screen_dimensions: [
                    (self.width * FONT_WIDTH) as f32,
                    (self.height * FONT_HEIGHT) as f32,
                ],
                frame_counter: self.frame_counter,
            },
        );

        self.encoder.clear(
            &self.screen_pipeline_data.out,
            [0.0, 0.2, 0.0, 1.0],
        );

        self.encoder.draw(
            &self.screen_vertex_slice,
            &self.screen_pipeline,
            &self.screen_pipeline_data,
        );

        self.encoder.flush(&mut self.device);

        self.window.gl_swap_window();
        self.device.cleanup();
        self.frame_counter += 1;

        let t = time::precise_time_ns();
        let dt = (t - self.last_render_time_ns) as f64;
        let new_fps = 1000000000.0 / dt;
        self.fps = 0.9 * self.fps + 0.1 * new_fps;
        self.last_render_time_ns = t;

        Ok(self.event_pump.poll_iter())
    }

    /// Update the character matrix with the provided data.
    pub fn update<T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<Instance>,
    {
        for (i, d) in self.instances.iter_mut().zip(data) {
            let d2: Instance = d.into();
            i.color = d2.color;
            i.bg_color = d2.bg_color;
            i.character = d2.character;
        }
    }

    /// Get the current frames per second. This is based on a rolling average, not the
    /// instantaneous measurement.
    pub fn get_fps(&self) -> f64 {
        self.fps
    }

    /// Get the number of frames that have been rendered.
    pub fn get_frame_counter(&self) -> u32 {
        self.frame_counter
    }
}

fn gfx_load_texture<F, R>(factory: &mut F) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    use gfx::format::Rgba8;
    let img = image::open("resources/12x12.png").unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory
        .create_texture_immutable_u8::<Rgba8>(kind, &[&img])
        .unwrap();
    view
}
