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
use log::{info, trace};
use std::convert::TryInto;

use crate::graphics::drawing::SpriteCell;
use crate::resources::sprite::SpriteTexture;
use bytemuck;
use wgpu::util::DeviceExt;

#[cfg(test)]
mod tests;

/// Error type for the renderer.
#[derive(Debug)]
pub enum RenderError {
    /// Generic error.
    GeneralError(String),
}

impl<S> std::convert::From<S> for RenderError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        RenderError::GeneralError(s.to_string())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CellGlobals {
    screen_size_in_sprites: [u32; 2],
    sprite_map_dimensions: [u32; 2],
    sprite_texture_dimensions: [u32; 2],
    sprite_dimensions: [u32; 2],
    palette_texture_dimensions: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenGlobals {
    screen_size: [f32; 2],
    scale_factor: [f32; 2],
    frame_counter: u32,
    elapsed_time: f32,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    translate: [f32; 2],
    cell_coords: [u32; 2],
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[u32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[u32; 2]>()
                        + std::mem::size_of::<u32>())
                        as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
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

/// Encapsulates the destination of the rendered output (SwapChain or texture).
enum RenderOutput {
    SwapChain {
        swap_chain: wgpu::SwapChain,
        swap_chain_format: wgpu::TextureFormat,
        swap_chain_descriptor: wgpu::SwapChainDescriptor,
        surface: wgpu::Surface,
        current_screen_size: winit::dpi::PhysicalSize<u32>,
    },
    Texture {
        texture: wgpu::Texture,
        texture_view: wgpu::TextureView,
        output_size: wgpu::Extent3d,
    },
}

impl RenderOutput {
    fn output_size(&self) -> (u32, u32) {
        match self {
            RenderOutput::SwapChain {
                current_screen_size: size,
                ..
            } => (size.width as _, size.height as _),
            RenderOutput::Texture {
                output_size: size, ..
            } => (size.width as _, size.height as _),
        }
    }

    fn output_format(&self) -> wgpu::TextureFormat {
        match self {
            RenderOutput::SwapChain {
                swap_chain_format, ..
            } => *swap_chain_format,
            RenderOutput::Texture { .. } => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

pub(crate) struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    render_output: RenderOutput,

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
    screen_uniform_buffer: wgpu::Buffer,
    screen_uniform_bind_group: wgpu::BindGroup,

    instances: Box<[Instance]>,
    palette_data: Box<[[[u8; 3]; 16]]>,
    palette_texture: wgpu::Texture,
    palette_texture_size: wgpu::Extent3d,

    pub(crate) pixel_dimensions: (u32, u32),
    pub(crate) aspect_ratio: (u32, u32),
    pub(crate) dimensions: (u32, u32),

    clear_color: wgpu::Color,

    last_render_time: time::OffsetDateTime,
    elapsed_time: time::Duration,
    frame_counter: u32,
    fps: f32,
}

impl Renderer {
    pub(crate) fn new(
        window: Option<&winit::window::Window>,
        dimensions: (u32, u32),
        sprite_texture: &SpriteTexture,
        clear_color: crate::resources::color::Color,
        screen_filter_method: wgpu::FilterMode,
        present_mode: wgpu::PresentMode,
    ) -> Result<Self, RenderError> {
        let mut instances = vec![Instance::default(); (dimensions.0 * dimensions.1) as usize];
        let palette_data = vec![[[255, 255, 255]; 16]; (dimensions.0 * dimensions.1) as usize];

        for y in 0..dimensions.1 {
            for x in 0..dimensions.0 {
                instances[(y * dimensions.0 + x) as usize] = Instance {
                    translate: [
                        -1.0 + (x as f32 * 2.0 / dimensions.0 as f32),
                        1.0 - ((y as f32 + 1.0) * 2.0 / dimensions.1 as f32),
                    ],
                    cell_coords: [x as _, y as _],
                    sprite: 0,
                    index: (y * dimensions.0 + x) as u32,
                };
            }
        }

        let screen_width = dimensions.0 * sprite_texture.sprite_width() as u32;
        let screen_height = dimensions.1 * sprite_texture.sprite_height() as u32;

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        // TODO: Determine whether this is portable. We definitely want Unorm, not Srgb, here.
        let swap_chain_format = wgpu::TextureFormat::Bgra8Unorm;

        let surface = window.map(|w| unsafe { instance.create_surface(w) });

        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: surface.as_ref(),
            }))
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        let render_target_size = wgpu::Extent3d {
            width: screen_width as _,
            height: screen_height as _,
            depth_or_array_layers: 1,
        };

        let render_output = surface.map_or_else(
            || {
                let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: render_target_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm, //swapchain_format,
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
                    label: Some("final output texture"),
                });

                let texture_view = output_texture.create_view(&Default::default());
                RenderOutput::Texture {
                    texture: output_texture,
                    texture_view,
                    output_size: render_target_size,
                }
            },
            |surface| {
                let sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: swap_chain_format,
                    width: screen_width as _,
                    height: screen_height as _,
                    present_mode,
                };

                let swap_chain = device.create_swap_chain(&surface, &sc_desc);

                RenderOutput::SwapChain {
                    swap_chain,
                    swap_chain_format,
                    swap_chain_descriptor: sc_desc,
                    surface,

                    current_screen_size: winit::dpi::PhysicalSize::<u32>::new(
                        screen_width as _,
                        screen_height as _,
                    ),
                }
            },
        );

        let render_target_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: render_target_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
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
            depth_or_array_layers: 1,
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
            trace!(
                "{:?}",
                &sprite_texture.pixels()[y * sprite_texture.width()
                    ..y * sprite_texture.width() + sprite_texture.width()]
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<String>()
            );
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &sprite_texture_gpu,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            sprite_texture.pixels(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: (sprite_texture.width() as u32).try_into().ok(),
                rows_per_image: (sprite_texture.height() as u32).try_into().ok(),
            },
            sprite_texture_size,
        );

        let sprite_texture_view = sprite_texture_gpu.create_view(&Default::default());

        let palette_texture_size = wgpu::Extent3d {
            width: dimensions.0 as u32 * 16u32,
            height: dimensions.1 as u32,
            depth_or_array_layers: 1,
        };

        let palette_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: palette_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("palette texture"),
        });

        let palette_texture_view = palette_texture.create_view(&Default::default());

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
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
                    resource: wgpu::BindingResource::TextureView(&palette_texture_view),
                },
            ],
            label: Some("cell_texture_bind_group"),
        });

        let cell_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Cell shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "render/shader/cell.wgsl"
            ))),
            flags: wgpu::ShaderFlags::all(),
        });

        let screen_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Screen shader"),
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
            sprite_texture_dimensions: [
                sprite_texture.width() as u32,
                sprite_texture.height() as u32,
            ],
            sprite_dimensions: [
                sprite_texture.sprite_width() as u32,
                sprite_texture.sprite_height() as u32,
            ],
            palette_texture_dimensions: [palette_texture_size.width, palette_texture_size.height],
        };

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
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

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
                targets: &[wgpu::TextureFormat::Rgba8Unorm.into()], //swapchain_format.into()],
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

        let screen_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Screen uniform bind group layout"),
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

        let screen_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screen uniform buffer"),
            size: std::mem::size_of::<ScreenGlobals>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen uniform bind group"),
            layout: &screen_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buffer.as_entire_binding(),
            }],
        });

        let screen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Screen pipeline layout"),
                bind_group_layouts: &[
                    &screen_texture_bind_group_layout,
                    &screen_uniform_bind_group_layout,
                ],
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
                    targets: &[render_output.output_format().into()],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            });

        // Calculate aspect ratio. This is used for letterboxing the screen when the window's
        // aspect ratio doesn't match.
        let (mut ax, mut ay) = (screen_width, screen_height);
        fn gcd(mut a: u32, mut b: u32) -> u32 {
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
        info!("swapchain format: {:?}", swap_chain_format);

        Ok(Renderer {
            device,
            queue,

            render_output,

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
            screen_uniform_buffer,
            screen_uniform_bind_group,

            instances: instances.into_boxed_slice(),
            instance_buffer,

            palette_data: palette_data.into_boxed_slice(),
            palette_texture,
            palette_texture_size,

            render_target_view,
            aspect_ratio: (ax, ay),
            pixel_dimensions: (screen_width as _, screen_height as _),
            dimensions,

            clear_color: clear_color.into(),
            last_render_time: time::OffsetDateTime::now_utc(),
            elapsed_time: time::Duration::zero(),
            frame_counter: 0,
            fps: 0.0,
        })
    }

    pub(crate) fn render_frame(&mut self) -> Result<(), RenderError> {
        let (screen_w, screen_h) = self.render_output.output_size();
        let (ax, ay) = self.aspect_ratio;
        let target_w = std::cmp::min(screen_w, (screen_h * ax) / ay);
        let target_h = std::cmp::min(screen_h, (screen_w * ay) / ax);

        let screen_uniforms = ScreenGlobals {
            screen_size: [screen_w as _, screen_h as _],
            frame_counter: self.frame_counter,
            elapsed_time: self.elapsed_time.as_seconds_f32(),
            scale_factor: [
                target_w as f32 / screen_w as f32,
                target_h as f32 / screen_h as f32,
            ],
        };

        // TODO: update palette data in-place in update() instead of making a copy here.
        let flat_palette_data: Vec<u8> = self
            .palette_data
            .iter()
            .flat_map(|c| c.iter())
            .map(|c| vec![c[0], c[1], c[2], 255])
            .flatten()
            .collect();

        self.queue.write_buffer(
            &self.screen_uniform_buffer,
            0,
            bytemuck::cast_slice(&[screen_uniforms]),
        );

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances[..]),
        );

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.palette_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &flat_palette_data[..],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: ((self.dimensions.0 * 4 * 16) as u32).try_into().ok(),
                rows_per_image: (self.dimensions.1 as u32).try_into().ok(),
            },
            self.palette_texture_size,
        );

        enum Output<'a> {
            SwapChain(wgpu::SwapChainTexture),
            Texture(&'a wgpu::TextureView),
        }

        impl<'a> Output<'a> {
            fn get(&'a self) -> &'a wgpu::TextureView {
                match self {
                    Output::SwapChain(sc_tex) => &sc_tex.view,
                    Output::Texture(tv) => tv,
                }
            }
        }

        let sc_tex = match &self.render_output {
            RenderOutput::SwapChain { swap_chain, .. } => {
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Couldn't get output frame")
                    .output;
                Output::SwapChain(frame)
            }
            RenderOutput::Texture { texture_view, .. } => Output::Texture(texture_view),
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main render encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main sprite cell pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.render_target_view,
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
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &sc_tex.get(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.screen_render_pipeline);
            render_pass.set_bind_group(0, &self.screen_texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.screen_uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.screen_vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.cell_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..QUAD_INDICES.len() as _, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));

        let t = time::OffsetDateTime::now_utc();
        let dt = t - self.last_render_time;
        let dt_micros = dt.whole_microseconds();
        if dt_micros > 0 {
            let new_fps = 1_000_000.0 / dt_micros as f32;
            self.fps = 0.9 * self.fps + 0.1 * new_fps;
            self.elapsed_time += dt;
        }
        if self.frame_counter % 1000 == 0 {
            info!("{} FPS", self.fps);
        }
        self.last_render_time = t;
        self.frame_counter += 1;

        Ok(())
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        match &mut self.render_output {
            RenderOutput::SwapChain {
                ref mut swap_chain,
                ref mut swap_chain_descriptor,
                ref mut current_screen_size,
                surface,
                ..
            } => {
                *current_screen_size = new_size;
                swap_chain_descriptor.width = new_size.width;
                swap_chain_descriptor.height = new_size.height;
                *swap_chain = self
                    .device
                    .create_swap_chain(&surface, &swap_chain_descriptor);
            }
            _ => {}
        };
    }

    pub(crate) fn fetch_render_output(&self) -> Option<Box<[u8]>> {
        if let RenderOutput::Texture {
            texture,
            output_size,
            ..
        } = &self.render_output
        {
            let unpadded_bytes_per_row = output_size.width * 4;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
            let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
            let download_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Render download buffer"),
                size: (padded_bytes_per_row * output_size.height) as u64,
                usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });
            {
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Main render encoder"),
                        });
                encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::ImageCopyBuffer {
                        buffer: &download_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: padded_bytes_per_row.try_into().ok(),
                            rows_per_image: None,
                        },
                    },
                    *output_size,
                );
                self.queue.submit(Some(encoder.finish()));
            }
            let download_slice = download_buffer.slice(..);
            let buffer_future = download_slice.map_async(wgpu::MapMode::Read);
            self.device.poll(wgpu::Maintain::Wait);
            futures::executor::block_on(buffer_future).expect("Couldn't download render output");
            let unpadded_image = download_slice.get_mapped_range()[..]
                .chunks(padded_bytes_per_row as usize)
                .flat_map(|row| row.iter().take(unpadded_bytes_per_row as usize))
                .cloned()
                .collect::<Vec<_>>();
            drop(download_slice);
            download_buffer.unmap();
            Some(unpadded_image.into_boxed_slice())
        } else {
            None
        }
    }

    /// Get the number of frames that have been rendered.
    fn get_frame_counter(&self) -> u32 {
        self.frame_counter
    }
}

impl RenderInterface for Renderer {
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
