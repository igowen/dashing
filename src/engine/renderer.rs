use gfx;
use gfx_core;
use image;
use std;
use time;

use engine::mid;

use gfx::traits::FactoryExt;

/// Color format required by the renderer.
pub type ColorFormat = gfx::format::Srgba8;
/// Depth format required by the renderer.
pub type DepthFormat = gfx::format::DepthStencil;

/// Error type for the renderer.
#[derive(Debug)]
pub enum RenderError {
    /// Generic error.
    GeneralError(String),
    /// Error from the OpenGL subsystem.
    OpenGLError(String),
}

impl<S> std::convert::From<S> for RenderError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        RenderError::GeneralError(s.to_string())
    }
}

// TODO: make these non-constant.
/// Font width.
pub const FONT_WIDTH: usize = 12;
/// Font height.
pub const FONT_HEIGHT: usize = 12;

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

mod internal {
    use gfx;
    gfx_defines!{
        // Individual vertices.
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
        }

        // Character cell index vertices.
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
            elapsed_time: f32 = "u_ElapsedTime",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            instance: gfx::InstanceBuffer<Instance> = (),
            tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
            screen_target: gfx::RenderTarget<super::super::ColorFormat> = "IntermediateTarget",
            locals: gfx::ConstantBuffer<Locals> = "Locals",
        }

        pipeline screen_pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            screen_tex: gfx::TextureSampler<[f32; 4]> = "t_ScreenTexture",
            locals: gfx::ConstantBuffer<ScreenLocals> = "Locals",
            out: gfx::RenderTarget<super::super::ColorFormat> = "Target0",
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
}

use self::internal::{Vertex, Instance, Locals, ScreenLocals, pipe, screen_pipe};

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

/// `Renderer` is responsible for rendering the grid of character cells. It's the lowest level
/// abstraction on top of the graphics subsystem, and you generally shouldn't need to interact with
/// it much, if at all, but it's still public just in case.
///
/// From a type perspective, this could fairly easily be made generic over different gfx backends,
/// but we would need some way of switching out the shader code, and I don't really anticipate
/// supporting any backend other than OpenGL.
pub struct Renderer<D, F>
where
    D: gfx::Device,
    F: gfx::Factory<D::Resources>,
{
    device: D,
    factory: F,
    encoder: gfx::Encoder<D::Resources, D::CommandBuffer>,

    color_view: gfx_core::handle::RenderTargetView<D::Resources, ColorFormat>,
    depth_view: gfx_core::handle::DepthStencilView<D::Resources, DepthFormat>,

    // GPU-side resources.
    vertex_slice: gfx::Slice<D::Resources>,
    screen_vertex_slice: gfx::Slice<D::Resources>,
    upload_buffer: gfx::handle::Buffer<D::Resources, Instance>,
    pipeline: gfx::pso::PipelineState<D::Resources, pipe::Meta>,
    screen_pipeline: gfx::pso::PipelineState<D::Resources, screen_pipe::Meta>,
    pipeline_data: pipe::Data<D::Resources>,
    screen_pipeline_data: screen_pipe::Data<D::Resources>,

    // CPU-side resources.
    width: usize,
    height: usize,
    instance_count: usize,
    instances: Box<[Instance]>,

    // Renderer metadata.
    frame_counter: u32,
    last_render_time_ns: u64,
    elapsed_time_ns: u64,
    fps: f32,
}

impl<D, F> Renderer<D, F>
where
    D: gfx::Device,
    F: gfx::Factory<D::Resources>,
{
    /// Create a new `Renderer` with the given device resources and dimensions, which are measured
    /// in characters.
    pub fn new(
        device: D,
        mut factory: F,
        command_buffer: D::CommandBuffer,
        color_view: gfx_core::handle::RenderTargetView<D::Resources, ColorFormat>,
        depth_view: gfx_core::handle::DepthStencilView<D::Resources, DepthFormat>,
        width: usize,
        height: usize,
    ) -> Result<Self, RenderError> {
        let encoder: gfx::Encoder<D::Resources, D::CommandBuffer> = command_buffer.into();

        let cell_pso: gfx::pso::PipelineState<D::Resources, pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("shader/cell.glslv"),
                include_bytes!("shader/cell.glslf"),
                pipe::new(),
            )?;

        let screen_pso: gfx::pso::PipelineState<D::Resources, screen_pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("shader/screen.glslv"),
                include_bytes!("shader/screen.glslf"),
                screen_pipe::new(),
            )?;

        let (cell_vertex_buffer, mut cell_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);
        let (screen_vertex_buffer, screen_slice) =
            factory.create_vertex_buffer_with_slice(&SCREEN_QUAD_VERTICES, &QUAD_INDICES[..]);
        let instance_count = width * height;

        cell_slice.instances = Some((instance_count as u32, 0));

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

        let screen_width = width * FONT_WIDTH;
        let screen_height = height * FONT_HEIGHT;

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
            vbuf: cell_vertex_buffer,
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

        Ok(Renderer {
            device: device,
            factory: factory,
            encoder: encoder,
            color_view: color_view,
            depth_view: depth_view,

            vertex_slice: cell_slice,
            screen_vertex_slice: screen_slice,
            upload_buffer: upload,
            pipeline: cell_pso,
            screen_pipeline: screen_pso,
            pipeline_data: intermediate_data,
            screen_pipeline_data: final_data,

            width: width,
            height: height,
            instance_count: instance_count,
            instances: instance_templates.into_boxed_slice(),

            frame_counter: 0,
            last_render_time_ns: 0,
            elapsed_time_ns: 0,
            fps: 0.0,
        })
    }

    /// Render one one frame.
    pub fn render(&mut self) -> Result<(), RenderError> {
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
                elapsed_time: self.elapsed_time_ns as f32 / 1_000_000_000.0,
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

        self.device.cleanup();

        self.frame_counter += 1;

        let t = time::precise_time_ns();
        if self.last_render_time_ns > 0 {
            let dt = (t - self.last_render_time_ns) as f32;
            let new_fps = 1_000_000_000.0 / dt;
            self.fps = 0.9 * self.fps + 0.1 * new_fps;
            self.elapsed_time_ns += t - self.last_render_time_ns;
        }
        self.last_render_time_ns = t;

        Ok(())
    }

    /// Get the current frames per second. This is based on a rolling average, not the
    /// instantaneous measurement.
    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    /// Get the number of frames that have been rendered.
    pub fn get_frame_counter(&self) -> u32 {
        self.frame_counter
    }
}

impl<'a, D, F> Renderer<D, F>
where
    D: gfx::Device,
    F: gfx::Factory<D::Resources>,
{
    /// Update the character matrix with the provided data.
    pub fn update<T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a mid::CharCell>,
    {
        for (i, d) in self.instances.iter_mut().zip(data) {
            let c: &mid::CharCell = d.into();
            i.color = c.fg_color;
            i.bg_color = c.bg_color;
            i.character = c.character;
        }
    }
}
