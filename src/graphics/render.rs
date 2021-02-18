// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use itertools;
#[allow(unused)]
use itertools::Itertools;
use log::info;

use crate::graphics::drawing::SpriteCell;
use crate::resources::sprite::SpriteTexture;
use bytemuck;
use wgpu::util::DeviceExt;

//#[cfg(test)]
//mod tests;

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

/*
// The gfx_defines! macro makes the output structs public, so we put them in an internal,
// non-public module to avoid cluttering up the public interface.
//
// NB: Scalars have looser alignment requirements in OpenGL, so we put them at the end of the struct
// definition to minimize the possibility for error.
mod internal {
    // Need both of these `use` statements due to the way the macros are written :/
    use gfx;
    use gfx::*;
    gfx_defines! {
        // Individual vertices.
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
        }

        // Character cell index vertices.
        vertex Instance {
            translate: [f32; 2] = "a_Translate",
            sprite_pos: [f32; 2] = "a_SpritePos",
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
            scale_factor: [f32; 2] = "u_ScaleFactor",
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
                sprite_pos: [0.0, 0.0],
                sprite: 0,
                index: 0,
            }
        }
    }
}

use self::internal::{pipe, screen_pipe, CellGlobals, Instance, ScreenGlobals, Vertex};
*/
/*
*/
/*
/// `Renderer` is responsible for rendering the grid of sprite cells. It's the lowest level
/// abstraction on top of the graphics subsystem.
///
/// From a type perspective, this could fairly easily be made generic over different gfx backends.
/// To do that, howerver, we would need some way of switching out the shader code, and I don't
/// really anticipate supporting any backend other than OpenGL.
pub(crate) struct Renderer<D, F>
where
    D: gfx::Device,
    F: gfx::Factory<D::Resources>,
{
    device: D,
    factory: F,
    encoder: gfx::Encoder<D::Resources, D::CommandBuffer>,

    pub(crate) depth_view: gfx_core::handle::DepthStencilView<D::Resources, DepthFormat>,

    // GPU-side resources.
    vertex_slice: gfx::Slice<D::Resources>,
    screen_vertex_slice: gfx::Slice<D::Resources>,
    upload_buffer: gfx::handle::Buffer<D::Resources, Instance>,
    pipeline: gfx::pso::PipelineState<D::Resources, pipe::Meta>,
    screen_pipeline: gfx::pso::PipelineState<D::Resources, screen_pipe::Meta>,
    pipeline_data: pipe::Data<D::Resources>,
    pub(crate) screen_pipeline_data: screen_pipe::Data<D::Resources>,

    // CPU-side resources.
    width: usize,
    height: usize,
    pub(crate) aspect_ratio: (usize, usize),
    pub(crate) sprite_width: usize,
    pub(crate) sprite_height: usize,
    instance_count: usize,
    instances: Box<[Instance]>,
    palette_texture: gfx::handle::Texture<D::Resources, gfx::format::R8_G8_B8_A8>,
    palette_data: Box<[[[u8; 3]; 16]]>,
    clear_color: [f32; 4],

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
    pub(crate) fn new<'a>(
        device: D,
        mut factory: F,
        command_buffer: D::CommandBuffer,
        color_view: gfx_core::handle::RenderTargetView<D::Resources, ColorFormat>,
        depth_view: gfx_core::handle::DepthStencilView<D::Resources, DepthFormat>,
        width: usize,
        height: usize,
        sprite_texture: &'a SpriteTexture,
        clear_color: [f32; 4],
        screen_filter_method: gfx::texture::FilterMethod,
    ) -> Result<Self, RenderError> {
        let encoder: gfx::Encoder<D::Resources, D::CommandBuffer> = command_buffer.into();

        let cell_pso: gfx::pso::PipelineState<D::Resources, pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("render/shader/cell.glslv"),
                include_bytes!("render/shader/cell.glslf"),
                pipe::new(),
            )?;

        // TODO: Allow users to provide their own pixel shader for full screen effects.
        let screen_pso: gfx::pso::PipelineState<D::Resources, screen_pipe::Meta> = factory
            .create_pipeline_simple(
                include_bytes!("render/shader/screen.glslv"),
                include_bytes!("render/shader/screen.glslf"),
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
            sprite_map_dimensions: [
                (sprite_texture.width() / sprite_texture.sprite_width()) as u32,
                (sprite_texture.height() / sprite_texture.sprite_height()) as u32,
            ],
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
                    sprite_pos: [x as f32, y as f32],
                    sprite: 0,
                    index: (y * width + x) as u32,
                }
            }
        }

        let upload = factory.create_upload_buffer::<Instance>(instance_count as usize)?;

        let cell_globals_buffer = factory.create_buffer_immutable(
            &[cell_globals],
            gfx::buffer::Role::Constant,
            gfx::memory::Bind::empty(),
        )?;

        let screen_globals_buffer = factory.create_constant_buffer(1);

        let screen_width = width * sprite_texture.sprite_width();
        let screen_height = height * sprite_texture.sprite_height();

        let (_, screen_texture, render_target) =
            factory.create_render_target(screen_width as u16, screen_height as u16)?;

        let screen_sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
            screen_filter_method,
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

        let palette_texture_kind = gfx::texture::Kind::D2(
            16 * width as u16,
            height as u16,
            gfx::texture::AaMode::Single,
        );

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
            gfx::memory::Bind::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic,
            Some(gfx::format::ChannelType::Unorm),
        )?;

        let palette_view = factory.view_texture_as_shader_resource::<gfx::format::Rgba8>(
            &palette_texture,
            (0, 0),
            gfx::format::Swizzle::new(),
        )?;

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
            out: color_view,
            globals: screen_globals_buffer,
        };

        Ok(Renderer {
            device: device,
            factory: factory,
            encoder: encoder,
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
            aspect_ratio: (ax, ay),
            sprite_width: sprite_texture.sprite_width(),
            sprite_height: sprite_texture.sprite_height(),
            instance_count: instance_count,
            instances: instance_templates.into_boxed_slice(),
            palette_texture: palette_texture,
            palette_data: palette_data.into_boxed_slice(),
            clear_color: clear_color,

            frame_counter: 0,
            last_render_time_ns: 0,
            elapsed_time_ns: 0,
            fps: 0.0,
        })
    }

    /// Render one frame.
    pub(crate) fn render(&mut self) -> Result<(), RenderError> {
        {
            let mut writer = self.factory.write_mapping(&self.upload_buffer)?;
            writer.copy_from_slice(&self.instances[..]);
        }

        // Clear with a bright color so it's extra-obvious if something goes wrong.
        self.encoder
            .clear(&self.pipeline_data.screen_target, [1.0, 0.0, 0.0, 1.0]);

        self.encoder.copy_buffer(
            &self.upload_buffer,
            &self.pipeline_data.instance_buffer,
            0,
            0,
            self.upload_buffer.len(),
        )?;

        let flat_palette_data: Vec<[u8; 4]> = self
            .palette_data
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

        self.encoder
            .draw(&self.vertex_slice, &self.pipeline, &self.pipeline_data);

        self.encoder
            .clear(&self.screen_pipeline_data.out, self.clear_color);

        let (screen_w, screen_h, _, _) = self.screen_pipeline_data.out.get_dimensions();
        let (ax, ay) = self.aspect_ratio;
        let target_w = std::cmp::min(screen_w as usize, (screen_h as usize * ax) / ay);
        let target_h = std::cmp::min(screen_h as usize, (screen_w as usize * ay) / ax);

        self.encoder.update_constant_buffer(
            &self.screen_pipeline_data.globals,
            &ScreenGlobals {
                screen_size: [
                    (self.width * self.sprite_width) as f32,
                    (self.height * self.sprite_height) as f32,
                ],
                frame_counter: self.frame_counter,
                elapsed_time: self.elapsed_time_ns as f32 / 1_000_000_000.0,
                scale_factor: [
                    target_w as f32 / screen_w as f32,
                    target_h as f32 / screen_h as f32,
                ],
            },
        );

        self.encoder.draw(
            &self.screen_vertex_slice,
            &self.screen_pipeline,
            &self.screen_pipeline_data,
        );

        self.encoder.flush(&mut self.device);

        self.device.cleanup();

        self.frame_counter += 1;

        // Cap FPS.
        // I *think* we will still continue to receive events during the sleep, but this will need
        // testing.
        // let tp = time::precise_time_ns();
        // let dtp = tp - self.last_render_time_ns;
        // let mi = (1.0 / 30.0 * 1_000_000_000.0) as u64;
        // if dtp < mi {
        //     std::thread::sleep(std::time::Duration::from_nanos(mi - dtp));
        // }

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

    /// Get the number of frames that have been rendered.
    fn get_frame_counter(&self) -> u32 {
        self.frame_counter
    }
}

impl<D, F> RenderInterface for Renderer<D, F>
where
    D: gfx::Device,
    F: gfx::Factory<D::Resources>,
{
    /// Update the sprite matrix with the provided data.
    fn update<'a, T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a SpriteCell>,
    {
        for (i, d, p) in itertools::multizip((
            self.instances.iter_mut(),
            data,
            self.palette_data.iter_mut(),
        )) {
            let c: &SpriteCell = d.into();
            i.sprite = c.sprite;
            *p = c.palette.into();
        }
    }

    /// Get the current frames per second. This is based on a rolling average, not an
    /// instantaneous measurement.
    fn get_fps(&self) -> f32 {
        self.fps
    }
}
*/

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CellGlobals {
    screen_size_in_sprites: [u32; 2],
    sprite_map_dimensions: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    translate: [f32; 2],
    sprite_pos: [f32; 2],
    sprite: u32,
    index: u32,
}

impl Instance {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<u32>())
                        as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint,
                },
            ],
        }
    }
}

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

// Vertices for the screen quad.
const SCREEN_QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [1.0, -1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, -1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
];

// Triangulation for the above vertices, shared by both the cell quads and the screen quad.
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub(crate) struct Renderer {
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    surface: wgpu::Surface,

    cell_render_pipeline: wgpu::RenderPipeline,

    cell_vertex_buffer: wgpu::Buffer,
    cell_index_buffer: wgpu::Buffer,

    cell_uniform_buffer: wgpu::Buffer,
    cell_uniform_bind_group: wgpu::BindGroup,

    cell_texture_bind_group: wgpu::BindGroup,

    render_target_view: wgpu::TextureView,

    instance_buffer: wgpu::Buffer,

    screen_render_pipeline: wgpu::RenderPipeline,

    screen_vertex_buffer: wgpu::Buffer,
    screen_index_buffer: wgpu::Buffer,
    screen_texture_bind_group: wgpu::BindGroup,

    instances: Box<[Instance]>,

    pub(crate) aspect_ratio: (usize, usize),

    last_render_time_ns: u64,
    elapsed_time_ns: u64,
    fps: f32,
}

impl Renderer {
    pub(crate) fn new(
        window: &winit::window::Window,
        dimensions: (usize, usize),
        sprite_texture: &SpriteTexture,
        clear_color: [f32; 4],
        screen_filter_method: wgpu::FilterMode,
    ) -> Result<Self, RenderError> {
        let mut instances = vec![Instance::default(); dimensions.0 * dimensions.1];
        for y in 0..dimensions.1 {
            for x in 0..dimensions.0 {
                instances[(y * dimensions.0 + x) as usize] = Instance {
                    translate: [
                        -1.0 + (x as f32 * 2.0 / dimensions.0 as f32),
                        1.0 - ((y as f32 + 1.0) * 2.0 / dimensions.1 as f32),
                    ],
                    sprite_pos: [x as _, y as _],
                    sprite: (('a' as u32 + (x + y) as u32) % 256) as _, //((x + y) % 64) as _,
                    index: (y * dimensions.0 + x) as u32,
                }
            }
        }

        let screen_width = dimensions.0 * sprite_texture.sprite_width();
        let screen_height = dimensions.1 * sprite_texture.sprite_height();
        //let instances = vec![Instance::default(); 200];

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            }))
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            Some(std::path::Path::new("./trace")),
            //None,
        ))
        .expect("Failed to create device");

        let swapchain_format = adapter.get_swap_chain_preferred_format(&surface);

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            // TODO: Mailbox -> Fifo
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let render_target_size = wgpu::Extent3d {
            width: screen_width as _,
            height: screen_height as _,
            depth: 1,
        };

        let render_target_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: render_target_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: swapchain_format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
            label: Some("render target texture"),
        });

        let render_target_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: screen_filter_method,
            min_filter: screen_filter_method,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_target_view = render_target_texture.create_view(&Default::default());

        let sprite_texture_size = wgpu::Extent3d {
            width: sprite_texture.width() as _,
            height: sprite_texture.height() as _,
            depth: 1,
        };

        let sprite_texture_gpu = device.create_texture(&wgpu::TextureDescriptor {
            size: sprite_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("sprite texture"),
        });

        for y in 0..sprite_texture.height() {
            info!(
                "{:?}",
                &sprite_texture.pixels()[y * sprite_texture.width()
                    ..y * sprite_texture.width() + sprite_texture.width()]
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<String>()
            );
        }

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &sprite_texture_gpu,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            sprite_texture.pixels(),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: sprite_texture.width() as u32,
                rows_per_image: sprite_texture.height() as u32,
            },
            sprite_texture_size,
        );

        let sprite_texture_view = sprite_texture_gpu.create_view(&Default::default());

        let sprite_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let cell_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("cell_texture_bind_group_layout"),
            });

        let cell_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cell_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sprite_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sprite_texture_sampler),
                },
            ],
            label: Some("cell_texture_bind_group"),
        });

        let cell_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "render/shader/cell.wgsl"
            ))),
            flags: wgpu::ShaderFlags::all(),
        });

        let screen_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "render/shader/screen.wgsl"
            ))),
            flags: wgpu::ShaderFlags::all(),
        });

        let cell_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cell vertex buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let cell_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cell index buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let cell_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cell uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let cell_uniforms = CellGlobals {
            screen_size_in_sprites: [dimensions.0 as _, dimensions.1 as _],
            sprite_map_dimensions: [
                (sprite_texture.width() / sprite_texture.sprite_width()) as u32,
                (sprite_texture.height() / sprite_texture.sprite_height()) as u32,
            ],
        };

        info!("{:?}", cell_uniforms);

        let cell_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cell uniform buffer"),
            contents: bytemuck::cast_slice(&[cell_uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let cell_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cell uniform bind group"),
            layout: &cell_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cell_uniform_buffer.as_entire_binding(),
            }],
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let cell_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell pipeline layout"),
            bind_group_layouts: &[
                &cell_uniform_bind_group_layout,
                &cell_texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let cell_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cell render pipeline"),
            layout: Some(&cell_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &cell_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), Instance::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &cell_shader,
                entry_point: "fs_main",
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        let screen_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen vertex buffer"),
            contents: bytemuck::cast_slice(&SCREEN_QUAD_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let screen_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen index buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let screen_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("screen_texture_bind_group_layout"),
            });

        let screen_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_target_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_target_sampler),
                },
            ],
            label: Some("screen_texture_bind_group"),
        });

        let screen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Screen pipeline layout"),
                bind_group_layouts: &[&screen_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let screen_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("screen pipeline"),
                layout: Some(&screen_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &screen_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &screen_shader,
                    entry_point: "fs_main",
                    targets: &[swapchain_format.into()],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            });

        // Calculate aspect ratio. This is used for letterboxing the screen when the window's
        // aspect ratio doesn't match.
        let (mut ax, mut ay) = (
            dimensions.0 * sprite_texture.sprite_width(),
            dimensions.1 * sprite_texture.sprite_height(),
        );
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            return a;
        }
        let g = gcd(ax, ay);
        ax /= g;
        ay /= g;

        info!("Aspect ratio: {}:{}", ax, ay);

        Ok(Renderer {
            swap_chain,
            swap_chain_descriptor: sc_desc,
            device,
            queue,
            surface,

            cell_render_pipeline,
            cell_vertex_buffer,
            cell_index_buffer,
            cell_uniform_buffer,
            cell_uniform_bind_group,
            cell_texture_bind_group,

            screen_render_pipeline,
            screen_vertex_buffer,
            screen_index_buffer,
            screen_texture_bind_group,

            instances: instances.into_boxed_slice(),
            instance_buffer,

            render_target_view,
            aspect_ratio: (ax, ay),

            last_render_time_ns: 0,
            elapsed_time_ns: 0,
            fps: 0.0,
        })
    }

    pub(crate) fn render_frame(&mut self) {
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture")
            .output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main sprite cell pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.render_target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.cell_render_pipeline);
            render_pass.set_bind_group(0, &self.cell_uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.cell_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.cell_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass
                .set_index_buffer(self.cell_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..self.instances.len() as _);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.5,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.screen_render_pipeline);
            render_pass.set_bind_group(0, &self.screen_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.screen_vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.cell_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));

        let t = time::precise_time_ns();
        if self.last_render_time_ns > 0 {
            let dt = (t - self.last_render_time_ns) as f32;
            let new_fps = 1_000_000_000.0 / dt;
            self.fps = 0.9 * self.fps + 0.1 * new_fps;
            self.elapsed_time_ns += t - self.last_render_time_ns;
        }
        self.last_render_time_ns = t;
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        //self.size = new_size;
        self.swap_chain_descriptor.width = new_size.width;
        self.swap_chain_descriptor.height = new_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}

impl RenderInterface for Renderer {
    /// Update the sprite matrix with the provided data.
    fn update<'a, T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a SpriteCell>,
    {
        for (i, d /*, p*/) in itertools::multizip((
            self.instances.iter_mut(),
            data,
            //self.palette_data.iter_mut(),
        )) {
            let c: &SpriteCell = d.into();
            i.sprite = c.sprite;
            //*p = c.palette.into();
        }
    }

    /// Get the current frames per second. This is based on a rolling average, not an
    /// instantaneous measurement.
    fn get_fps(&self) -> f32 {
        self.fps
    }
}

/// Interface for EngineDriver -> Renderer communication.
pub trait RenderInterface {
    /// Update the sprite matrix with the provided data.
    fn update<'a, T, U>(&mut self, data: T)
    where
        T: Iterator<Item = U>,
        U: Into<&'a SpriteCell>;

    /// Get the current FPS
    fn get_fps(&self) -> f32;
}
