#![deny(warnings)]
#![allow(dead_code)]

#[macro_use]
extern crate log;
extern crate pretty_logger;
extern crate time;

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate image;
extern crate sdl2;

use gfx::Device;
use gfx::Factory;
use gfx::traits::FactoryExt;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

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

//const QUAD_VERTICES: [Vertex; 4] = [
//    Vertex {
//        pos: [0.5, -0.5],
//        uv: [1.0, 1.0],
//    },
//    Vertex {
//        pos: [-0.5, -0.5],
//        uv: [0.0, 1.0],
//    },
//    Vertex {
//        pos: [-0.5, 0.5],
//        uv: [0.0, 0.0],
//    },
//    Vertex {
//        pos: [0.5, 0.5],
//        uv: [1.0, 0.0],
//    },
//];
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

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
// Screen dimensions in characters.
const WIDTH: u32 = 100;
const HEIGHT: u32 = 50;

const FONT_WIDTH: u32 = 12;
const FONT_HEIGHT: u32 = 12;

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
#[allow(dead_code)]
fn hsv2rgb(hsv: [f32; 4]) -> [f32; 4] {
    if hsv[1] <= 0.0 {
        // < is bogus, just shuts up warnings
        return [hsv[2], hsv[2], hsv[2], hsv[3]];
    }
    let mut hh = hsv[0];
    if hh >= 360.0 {
        hh = 0.0;
    }
    hh /= 60.0;
    let i = hh as i32;
    let ff = hh - i as f32;
    let p = hsv[2] * (1.0 - hsv[1]);
    let q = hsv[2] * (1.0 - (hsv[1] * ff));
    let t = hsv[2] * (1.0 - (hsv[1] * (1.0 - ff)));

    match i {
        0 => [hsv[2], t, p, hsv[3]],
        1 => [q, hsv[2], p, hsv[3]],
        2 => [p, hsv[2], t, hsv[3]],
        3 => [p, q, hsv[2], hsv[3]],
        4 => [t, p, hsv[2], hsv[3]],
        _ => [hsv[2], p, q, hsv[3]],
    }
}

// TODO(igowen): should this be generic over resource types?
struct LLEngine {
    // Handles to device resources we need to hold onto.
    #[allow(dead_code)]
    sdl_context: sdl2::Sdl,
    #[allow(dead_code)]
    video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    #[allow(dead_code)]
    gl_context: sdl2::video::GLContext,
    device: gfx_window_sdl::Device,
    factory: gfx_window_sdl::Factory,
    #[allow(dead_code)]
    color_view: gfx_core::handle::RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
    #[allow(dead_code)]
    depth_view: gfx_core::handle::DepthStencilView<gfx_device_gl::Resources, DepthFormat>,
    pipeline: gfx::pso::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    #[allow(dead_code)]
    screen_pipeline: gfx::pso::PipelineState<gfx_device_gl::Resources, screen_pipe::Meta>,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,

    // GPU-side resources.
    vertex_slice: gfx::Slice<gfx_device_gl::Resources>,
    screen_vertex_slice: gfx::Slice<gfx_device_gl::Resources>,
    upload_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Instance>,
    pipeline_data: pipe::Data<gfx_device_gl::Resources>,
    #[allow(dead_code)]
    screen_pipeline_data: screen_pipe::Data<gfx_device_gl::Resources>,

    // CPU-side resources.
    width: u32,
    height: u32,
    #[allow(dead_code)]
    instance_count: u32,
    instances: Box<[Instance]>,
    frame_counter: u32,
}

#[derive(Debug)]
enum LLEngineError {
    GeneralError(String),
}

impl<S> std::convert::From<S> for LLEngineError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        LLEngineError::GeneralError(s.to_string())
    }
}

impl LLEngine {
    pub fn new(width: u32, height: u32) -> Result<Self, LLEngineError> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        {
            let gl = video.gl_attr();
            gl.set_context_profile(sdl2::video::GLProfile::Core);
            gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        }

        let screen_width = width * FONT_WIDTH * 2;
        let screen_height = height * FONT_HEIGHT * 2;

        let builder = video.window("rlb", screen_width, screen_height);
        let window_result = gfx_window_sdl::init::<ColorFormat, DepthFormat>(builder);
        let (window, gl_context, device, mut factory, color_view, depth_view);
        match window_result {
            // TODO: fix this
            Err(e) => {
                return Err(LLEngineError::GeneralError(
                    format!("SDL init error: {:?}", e),
                ));
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
                    color: hsv2rgb([360.0 - (x as f32 / width as f32) * 360.0, 1.0, 1.0, 1.0]), //[1.0, 1.0, 1.0, 1.0], //hsv2rgb([((y * WIDTH + x) % 360) as f32, 1.0, 1.0, 1.0]),
                    bg_color: [0.0, 0.0, 0.0, 1.0], //hsv2rgb([(x as f32 / width as f32) * 360.0, 0.9, 0.5, 1.0]), //[0.0, 0.0, 0.0, 1.0],
                    character: (x + y) % 256,
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

        Ok(LLEngine {
            sdl_context: sdl_context,
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
        })
    }

    pub fn render(&mut self) -> Result<(), LLEngineError> {
        {
            let mut writer = self.factory.write_mapping(&self.upload_buffer)?;
            writer.copy_from_slice(&self.instances[..]);
            //if self.i % 10 == 0 {
            //    for x in 0..self.width {
            //        for y in 0..self.height {
            //            self.instances[(y * self.width + x) as usize].character =
            //                (x + y + self.i / 10) % 256;
            //        }
            //    }
            //}
            //writer[420].character = 1;
            //writer[420].color = [0.0, 0.0, 0.0, 1.0];
            //writer[420].bg_color = [0.0, 1.0, 0.0, 1.0];
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
        ); // draw commands with buffer data and attached pso

        self.encoder.update_constant_buffer(
            &self.screen_pipeline_data.locals,
            &ScreenLocals { frame_counter: self.frame_counter },
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

        self.encoder.flush(&mut self.device); // execute draw commands

        self.window.gl_swap_window();
        self.device.cleanup();
        self.frame_counter += 1;

        Ok(())
    }

    pub fn pump(&mut self) -> Result<bool, LLEngineError> {
        let mut event_pump = self.sdl_context.event_pump()?;

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    return Ok(true);
                }
                _ => {}
            }
        }
        return Ok(false);
    }
}

pub fn main() {
    pretty_logger::init_to_defaults().unwrap();
    info!("starting up");

    let mut engine = LLEngine::new(WIDTH, HEIGHT).unwrap();

    let mut fps = 0.0;
    let mut frame = 0;
    let mut last = time::precise_time_ns();
    'main: loop {
        engine.render().unwrap();
        if engine.pump().unwrap() {
            break 'main;
        }

        frame += 1;
        if frame > 120 {
            frame = 0;
            info!("{:.0} fps", fps);
        }

        let t = time::precise_time_ns();
        let dt = (t - last) as f64;
        let new_fps = 1000000000.0 / dt;
        fps = 0.9 * fps + 0.1 * new_fps;
        last = t;
    }

    //info!("clean shutdown.");
}
