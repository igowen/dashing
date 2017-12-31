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
    // TODO: load using resource module instead of directly.
    let img = image::open("resources/12x12.png").unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory
        .create_texture_immutable_u8::<Rgba8>(kind, gfx::texture::Mipmap::Provided, &[&img])
        .unwrap();
    view
}

// The gfx_defines! macro makes the output structs public, so we put them in an internal,
// non-public module to avoid cluttering up the public interface.
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

        // Uniforms for the character cell PSO.
        constant Locals {
            dim: [f32; 2] = "u_ScreenCharDim",
            font_dim: [f32; 2] = "u_FontCharDim",
        }

        // Uniforms for the screen PSO.
        constant ScreenLocals {
            screen_dimensions: [f32; 2] = "u_ScreenDimensions",
            frame_counter: u32 = "u_FrameCounter",
            elapsed_time: f32 = "u_ElapsedTime",
        }

        // Character cell pipeline.
        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            instance: gfx::InstanceBuffer<Instance> = (),
            tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
            screen_target: gfx::RenderTarget<super::super::ColorFormat> = "IntermediateTarget",
            locals: gfx::ConstantBuffer<Locals> = "Locals",
        }

        // Final screen pipeline.
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
            gfx::memory::Bind::TRANSFER_DST,
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
            gfx::memory::Bind::empty(),
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

// TODO: Probably want to move this into its own file since there's so much support code.
#[cfg(test)]
mod tests {
    // The output of the renderer is intended to be pixel-perfect, so the tests are written with
    // that in mind.
    use euclid::Size2D;
    use gfx::Factory;
    use gfx_core::memory::Typed;
    use gfx_device_gl;
    use gleam::gl::GlType;
    use offscreen_gl_context::{ColorAttachmentType, GLContext, NativeGLContext,
                               NativeGLContextMethods, GLVersion, GLContextAttributes};
    #[allow(unused)]
    use pretty_logger;
    use spectral::prelude::*;
    use std::os::raw::c_void;
    use std::sync::{Mutex, MutexGuard};
    use super::*;

    // offscreen_gl_context doesn't like it when multiple threads try to create a GL context
    // simultaneously, so we use a mutex to serialize the tests.
    lazy_static!{
        static ref OFFSCREEN_GL_MUTEX: Mutex<()> = Mutex::new(());
    }

    struct RenderTestSupportHarness {
        gl_context: GLContext<NativeGLContext>,
        renderer: Renderer<gfx_device_gl::Device, gfx_device_gl::Factory>,
        render_texture: gfx::handle::Texture<gfx_device_gl::Resources, gfx::format::R8_G8_B8_A8>,
        width: usize,
        height: usize,
        guard: MutexGuard<'static, ()>,
    }

    impl RenderTestSupportHarness {
        fn new(width: usize, height: usize) -> RenderTestSupportHarness {
            let guard = OFFSCREEN_GL_MUTEX.lock().unwrap();
            // Ideally we'd be able to use OSMesa here, but i couldn't get it to successfully create
            // a context with a GL version > 3.0.
            let gl_context = GLContext::<NativeGLContext>::new(
                Size2D::new(
                    (width * super::FONT_WIDTH) as i32,
                    (height * super::FONT_HEIGHT) as i32,
                ),
                GLContextAttributes::any(),
                ColorAttachmentType::Texture,
                GlType::Gl,
                GLVersion::MajorMinor(3, 2),
                None,
            ).unwrap();

            let (device, mut factory) =
                gfx_device_gl::create(|p| NativeGLContext::get_proc_address(p) as *const c_void);

            // Set up a texture to use as the render target. We can't just use
            // gfx::Factory::create_render_target because we need to set the TRANSFER_SRC bind flag
            // so we can copy the data back to CPU space.
            let texture: gfx::handle::Texture<_, gfx::format::R8_G8_B8_A8> = factory
                .create_texture(
                    gfx::texture::Kind::D2(
                        (width * super::FONT_WIDTH) as u16,
                        (height * super::FONT_HEIGHT) as u16,
                        gfx::texture::AaMode::Single,
                    ),
                    1,
                    gfx::memory::Bind::RENDER_TARGET | gfx::memory::Bind::SHADER_RESOURCE |
                        gfx::memory::Bind::TRANSFER_SRC,
                    gfx::memory::Usage::Data,
                    Some(gfx::format::ChannelType::Srgb),
                )
                .unwrap();
            // Get a render target view of the texture.
            let render_target = factory
                .view_texture_as_render_target(&texture, 0, None)
                .unwrap();

            let command_buffer = factory.create_command_buffer();
            let depth_view = factory
                .create_depth_stencil_view_only(
                    (width * FONT_WIDTH) as u16,
                    (height * FONT_HEIGHT) as u16,
                )
                .unwrap();
            let renderer = Renderer::new(
                device,
                factory,
                command_buffer,
                render_target,
                depth_view,
                width,
                height,
            ).unwrap();

            RenderTestSupportHarness {
                gl_context: gl_context,
                renderer: renderer,
                render_texture: texture,
                width: width,
                height: height,
                guard: guard,
            }
        }

        fn extract_render_result(&mut self) -> Vec<u8> {
            // Set up a download buffer to retrieve the image from the GPU.
            let info = self.render_texture.get_info().to_raw_image_info(
                gfx::format::ChannelType::Srgb,
                0,
            );
            let buffer = self.renderer
                .factory
                .create_download_buffer::<u8>(info.get_byte_count())
                .unwrap();

            // Copy the texture to the download buffer, and flush the command buffer.
            self.renderer
                .encoder
                .copy_texture_to_buffer_raw(
                    &self.render_texture.raw(),
                    None,
                    info,
                    &buffer.raw(),
                    0,
                )
                .unwrap();
            self.renderer.encoder.flush(&mut self.renderer.device);

            // Finally, read the rendered image from the download buffer.
            let reader = self.renderer.factory.read_mapping(&buffer).unwrap();
            let data = reader.to_vec();
            let mut flipped_image: Vec<u8> = vec![];

            // OpenGL stores textures upside down from what we would expect, so flip it before doing
            // the comparison.
            for row in data.chunks(FONT_WIDTH * self.width * 4).rev() {
                flipped_image.extend(row);
            }

            flipped_image
        }
    }

    #[test]
    fn render_one_cell() {
        let mut harness = RenderTestSupportHarness::new(1, 1);

        harness.renderer.update(
            [
                mid::CharCell {
                    fg_color: [1.0, 1.0, 1.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 1,
                    transparent: false,
                },
            ].iter(),
        );

        // Render the frame.
        harness.renderer.render().unwrap();

        let actual_image = harness.extract_render_result();

        let expected_image = image::load_from_memory(include_bytes!("testdata/one_cell.png"))
            .unwrap()
            .to_rgba()
            .into_raw();
        assert_that(&actual_image).is_equal_to(&expected_image);
    }

    #[test]
    fn render_one_cell_character_change() {
        let mut harness = RenderTestSupportHarness::new(1, 1);

        harness.renderer.update(
            [
                mid::CharCell {
                    fg_color: [1.0, 1.0, 0.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 2,
                    transparent: false,
                },
            ].iter(),
        );

        // Render the frame.
        harness.renderer.render().unwrap();

        let actual_image = harness.extract_render_result();

        let expected_image = image::load_from_memory(include_bytes!("testdata/one_cell.png"))
            .unwrap()
            .to_rgba()
            .into_raw();

        // These shouldn't match.
        assert_that(&actual_image).is_not_equal_to(&expected_image);

        harness.renderer.update(
            [
                mid::CharCell {
                    fg_color: [1.0, 1.0, 1.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 1,
                    transparent: false,
                },
            ].iter(),
        );

        // Render the frame.
        harness.renderer.render().unwrap();

        let actual_image_2 = harness.extract_render_result();

        // These should.
        assert_that(&actual_image_2).is_equal_to(&expected_image);
    }

    #[test]
    fn render_2x2_with_color() {
        let mut harness = RenderTestSupportHarness::new(2, 2);

        harness.renderer.update(
            [
                mid::CharCell {
                    fg_color: [1.0, 0.0, 1.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 72,
                    transparent: false,
                },
                mid::CharCell {
                    fg_color: [0.0, 1.0, 1.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 105,
                    transparent: false,
                },
                mid::CharCell {
                    fg_color: [1.0, 1.0, 0.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 33,
                    transparent: false,
                },
                mid::CharCell {
                    fg_color: [0.0, 1.0, 0.0, 1.0],
                    bg_color: [0.0, 0.0, 0.0, 1.0],
                    character: 19,
                    transparent: false,
                },
            ].iter(),
        );

        // Render the frame.
        harness.renderer.render().unwrap();

        let actual_image = harness.extract_render_result();

        let expected_image = image::load_from_memory(include_bytes!("testdata/hi.png"))
            .unwrap()
            .to_rgba()
            .into_raw();

        assert_that(&actual_image).is_equal_to(&expected_image);
    }
}
