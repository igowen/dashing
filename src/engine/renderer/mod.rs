use gfx;
use gfx_core;
use itertools;
use std;
use time;

use gfx::traits::FactoryExt;
#[allow(unused)]
use itertools::Itertools;

use resources::sprite::{Palette, SpriteTexture};

#[cfg(test)]
mod tests;

/// Color format required by the renderer.
pub type ColorFormat = gfx::format::Rgba8;
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

// The gfx_defines! macro makes the output structs public, so we put them in an internal,
// non-public module to avoid cluttering up the public interface.
//
// NB: Scalars have looser alignment requirements in OpenGL, so we put them at the end of the struct
// definition to minimize the possibility for error.
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
            sprite: u32 = "a_Sprite",
            index: u32 = "a_Index",
        }

        // Uniforms for the sprite cell PSO.
        constant CellGlobals {
            screen_size_in_sprites: [u32; 2] = "u_ScreenSizeInSprites",
            sprite_map_dimensions: [u32; 2] = "u_SpriteMapDimensions",
        }

        // Uniforms for the screen PSO.
        constant ScreenGlobals {
            screen_size: [f32; 2] = "u_ScreenSizeInPixels",
            frame_counter: u32 = "u_FrameCounter",
            elapsed_time: f32 = "u_ElapsedTime",
        }

        // Character cell pipeline.
        pipeline pipe {
            vertex_buffer: gfx::VertexBuffer<Vertex> = (),
            instance_buffer: gfx::InstanceBuffer<Instance> = (),
            sprite_texture: gfx::TextureSampler<u32> = "t_SpriteTexture",
            palette_texture: gfx::TextureSampler<[f32; 4]> = "t_Palette",
            screen_target: gfx::RenderTarget<super::super::ColorFormat> = "IntermediateTarget",
            globals: gfx::ConstantBuffer<CellGlobals> = "CellGlobals",
        }

        // Final screen pipeline.
        pipeline screen_pipe {
            vertex_buffer: gfx::VertexBuffer<Vertex> = (),
            screen_texture: gfx::TextureSampler<[f32; 4]> = "t_ScreenTexture",
            globals: gfx::ConstantBuffer<ScreenGlobals> = "ScreenGlobals",
            out: gfx::RenderTarget<super::super::ColorFormat> = "Target0",
        }
    }

    impl Default for Instance {
        fn default() -> Self {
            Instance {
                translate: [0.0, 0.0],
                sprite: 0,
                index: 0,
            }
        }
    }
}

use self::internal::{Vertex, Instance, CellGlobals, ScreenGlobals, pipe, screen_pipe};

// Vertices for sprite cell quads.
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

/// `Renderer` is responsible for rendering the grid of sprite cells. It's the lowest level
/// abstraction on top of the graphics subsystem, and you generally shouldn't need to interact with
/// it much, if at all, but it's still public just in case.
///
/// From a type perspective, this could fairly easily be made generic over different gfx backends.
/// To do that, howerver, we would need some way of switching out the shader code, and I don't
/// really anticipate supporting any backend other than OpenGL.
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
    sprite_width: usize,
    sprite_height: usize,
    instance_count: usize,
    instances: Box<[Instance]>,
    palette_texture: gfx::handle::Texture<D::Resources, gfx::format::R8_G8_B8_A8>,
    palette_data: Box<[[[u8; 3]; 16]]>,

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
    /// in sprites.
    pub fn new<'a>(
        device: D,
        mut factory: F,
        command_buffer: D::CommandBuffer,
        color_view: gfx_core::handle::RenderTargetView<D::Resources, ColorFormat>,
        depth_view: gfx_core::handle::DepthStencilView<D::Resources, DepthFormat>,
        width: usize,
        height: usize,
        sprite_texture: &'a SpriteTexture,
    ) -> Result<Self, RenderError> {
        let encoder: gfx::Encoder<D::Resources, D::CommandBuffer> = command_buffer.into();

        let cell_pso: gfx::pso::PipelineState<D::Resources, pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("shader/cell.glslv"),
                include_bytes!("shader/cell.glslf"),
                pipe::new(),
            )?;

        // TODO: Allow users to provide their own pixel shader for full screen effects.
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

        let cell_globals = CellGlobals {
            screen_size_in_sprites: [width as u32, height as u32],
            sprite_map_dimensions: [16, 16],
        };

        let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let instance_buffer = factory.create_buffer(
            instance_count as usize,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Data,
            gfx::memory::Bind::TRANSFER_DST,
        )?;

        let mut instance_templates = vec![Instance::default(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                instance_templates[(y * width + x) as usize] = Instance {
                    translate: [
                        -1.0 + (x as f32 * 2.0 / width as f32),
                        1.0 - ((y as f32 + 1.0) * 2.0 / height as f32),
                    ],
                    sprite: 0,
                    index: (y * width + x) as u32,
                }
            }
        }

        let upload = factory.create_upload_buffer::<Instance>(
            instance_count as usize,
        )?;

        let cell_globals_buffer = factory.create_buffer_immutable(
            &[cell_globals],
            gfx::buffer::Role::Constant,
            gfx::memory::Bind::empty(),
        )?;

        let screen_globals_buffer = factory.create_constant_buffer(1);

        let screen_width = width * sprite_texture.sprite_width();
        let screen_height = height * sprite_texture.sprite_height();

        let (_, screen_texture, render_target) = factory.create_render_target(
            screen_width as u16,
            screen_height as u16,
        )?;

        let screen_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Scale,
            gfx::texture::WrapMode::Clamp,
        ));

        let sprite_texture_kind = gfx::texture::Kind::D2(
            sprite_texture.width() as u16,
            sprite_texture.height() as u16,
            gfx::texture::AaMode::Single,
        );

        let (_, sprite_texture_view) = factory
            .create_texture_immutable_u8::<(gfx::format::R8, gfx::format::Uint)>(
                sprite_texture_kind,
                gfx::texture::Mipmap::Provided,
                &[&sprite_texture.pixels()],
            )?;

        let palette_texture_kind =
            gfx::texture::Kind::D2(16, instance_count as u16, gfx::texture::AaMode::Single);

        let mut palette_data = vec![[[255, 0, 255]; 16]; instance_count];

        for y in 0..height {
            for x in 0..width {
                palette_data[y * width + x][0] = [
                    ((x as f32 / width as f32) * 255.0) as u8,
                    0,
                    ((y as f32 / height as f32) * 255.0) as u8,
                ];
            }
        }

        let palette_texture = factory.create_texture::<gfx::format::R8_G8_B8_A8>(
            palette_texture_kind,
            1,
            //gfx::memory::Bind::TRANSFER_DST |
            gfx::memory::Bind::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic,
            Some(gfx::format::ChannelType::Unorm),
        )?;
        let palette_view = factory
            .view_texture_as_shader_resource::<gfx::format::Rgba8>(
                &palette_texture,
                (0, 0),
                gfx::format::Swizzle::new(),
            )?;

        /*
        let (_palette_texture, palette_view) = factory
            .create_texture_immutable_u8::<gfx::format::Rgba8>(
                kind,
                gfx::texture::Mipmap::Provided,
                &[&palette_data[..]],
            )?;*/

        let intermediate_data = pipe::Data {
            vertex_buffer: cell_vertex_buffer,
            instance_buffer: instance_buffer,
            sprite_texture: (sprite_texture_view, sampler.clone()),
            palette_texture: (palette_view, sampler.clone()),
            screen_target: render_target,
            globals: cell_globals_buffer,
        };

        let final_data = screen_pipe::Data {
            vertex_buffer: screen_vertex_buffer,
            screen_texture: (screen_texture, screen_sampler),
            out: color_view.clone(),
            globals: screen_globals_buffer,
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
            sprite_width: sprite_texture.sprite_width(),
            sprite_height: sprite_texture.sprite_height(),
            instance_count: instance_count,
            instances: instance_templates.into_boxed_slice(),
            palette_texture: palette_texture,
            palette_data: palette_data.into_boxed_slice(),

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
            [0.1, 0.0, 0.0, 1.0],
        );

        self.encoder.copy_buffer(
            &self.upload_buffer,
            &self.pipeline_data.instance_buffer,
            0,
            0,
            self.upload_buffer.len(),
        )?;

        let flat_palette_data: Vec<[u8; 4]> = self.palette_data
            .iter()
            .flat_map(|c| c.iter())
            .map(|c| [c[0], c[1], c[2], 255])
            .collect();
        let palette_info = self.palette_texture.get_info().to_image_info(0);

        self.encoder
            .update_texture::<gfx::format::R8_G8_B8_A8, gfx::format::Rgba8>(
                &self.palette_texture,
                None,
                palette_info,
                &flat_palette_data[..],
            )
            .unwrap();


        self.encoder.draw(
            &self.vertex_slice,
            &self.pipeline,
            &self.pipeline_data,
        );

        self.encoder.update_constant_buffer(
            &self.screen_pipeline_data.globals,
            &ScreenGlobals {
                screen_size: [
                    (self.width * self.sprite_width) as f32,
                    (self.height * self.sprite_height) as f32,
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

    /// Update the sprite matrix with the provided data.
    pub fn update<'a, T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a SpriteCellMeta>,
    {
        for (i, d, p) in itertools::multizip((
            self.instances.iter_mut(),
            data,
            self.palette_data.iter_mut(),
        ))
        {
            let c: &SpriteCellMeta = d.into();
            i.sprite = c.sprite;
            *p = c.palette.into();
        }
    }
}

/// Sprite metadata
#[derive(Copy, Clone, Default, Debug)]
pub struct SpriteCellMeta {
    /// Color for the cell.
    pub palette: Palette,
    /// Sprite index.
    pub sprite: u32,
}
