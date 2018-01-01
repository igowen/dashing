#[cfg(test)]
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

/// There's a lot of boilerplate in setting up the offscreen renderer and extracting the rendered
/// image, so we use a separate support harness to manage that.
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
        // Get an exclusive lock on the global offscreen GL mutex, so only one test case can be
        // running at a time.
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
            .copy_texture_to_buffer_raw(&self.render_texture.raw(), None, info, &buffer.raw(), 0)
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
