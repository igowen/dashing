#[cfg(test)]
// The output of the renderer is intended to be pixel-perfect, so the tests are written with
// that in mind.
use euclid::Size2D;
use gfx::Factory;
use gfx_core::memory::Typed;
use gfx_device_gl;
use gleam::gl::GlType;
use image;
use offscreen_gl_context::{ColorAttachmentType, GLContext, NativeGLContext,
                           NativeGLContextMethods, GLVersion, GLContextAttributes};
use png::{self, HasParameters};
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
    sprite_width: usize,
    sprite_height: usize,
    width: usize,
    height: usize,
    guard: MutexGuard<'static, ()>,
}

impl RenderTestSupportHarness {
    fn new(width: usize, height: usize) -> RenderTestSupportHarness {
        // Get an exclusive lock on the global offscreen GL mutex, so only one test case can be
        // running at a time.
        let guard = OFFSCREEN_GL_MUTEX.lock().unwrap();

        // Load the test sprite texture.
        // TODO: get rid of this once the sprite-loading code is done.
        let img = include_bytes!("testdata/12x12.png");
        let mut decoder = png::Decoder::new(&img[..]);
        // Need to set this so the index values don't get converted to RGBA.
        decoder.set(png::Transformations::IDENTITY);
        let (_, mut reader) = decoder.read_info().unwrap();
        let mut imgdata = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut imgdata[..]).unwrap();
        let tex = SpriteTexture::new_from_pixels(
            &imgdata[..],
            reader.info().size().0 as usize,
            reader.info().size().1 as usize,
            reader.info().size().0 as usize / 16,
            reader.info().size().1 as usize / 16,
            256,
        ).unwrap();

        let sprite_width = tex.sprite_width();
        let sprite_height = tex.sprite_height();

        // Ideally we'd be able to use OSMesa here, but i couldn't get it to successfully create
        // a context with a GL version > 3.0.
        let gl_context = GLContext::<NativeGLContext>::new(
            Size2D::new(
                (width * sprite_width) as i32,
                (height * sprite_height) as i32,
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
                    (width * sprite_width) as u16,
                    (height * sprite_height) as u16,
                    gfx::texture::AaMode::Single,
                ),
                1,
                gfx::memory::Bind::RENDER_TARGET | gfx::memory::Bind::SHADER_RESOURCE |
                    gfx::memory::Bind::TRANSFER_SRC,
                gfx::memory::Usage::Data,
                Some(gfx::format::ChannelType::Unorm),
            )
            .unwrap();
        // Get a render target view of the texture.
        let render_target = factory
            .view_texture_as_render_target(&texture, 0, None)
            .unwrap();

        let command_buffer = factory.create_command_buffer();
        let depth_view = factory
            .create_depth_stencil_view_only(
                (width * sprite_width) as u16,
                (height * sprite_height) as u16,
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
            &tex,
        ).unwrap();

        RenderTestSupportHarness {
            gl_context: gl_context,
            renderer: renderer,
            render_texture: texture,
            width: width,
            height: height,
            sprite_width: sprite_width,
            sprite_height: sprite_height,
            guard: guard,
        }
    }

    /// Extract the rendered image from the offscreen context.
    fn extract_render_result(&mut self) -> Vec<u8> {
        // Set up a download buffer to retrieve the image from the GPU.
        let info = self.render_texture.get_info().to_raw_image_info(
            gfx::format::ChannelType::Unorm,
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

        // OpenGL stores textures upside down from what we would expect, so flip it before doing
        // the comparison.
        let mut flipped_image = Vec::<u8>::with_capacity(
            self.sprite_width * self.width * self.sprite_height *
                self.height * 4,
        );
        for row in data.chunks(self.sprite_width * self.width * 4).rev() {
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
            SpriteCellMeta {
                palette: Palette::mono([255, 255, 255]).set(0, [0, 0, 0]),
                sprite: 1,
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
fn render_one_cell_sprite_change() {
    let mut harness = RenderTestSupportHarness::new(1, 1);

    harness.renderer.update(
        [
            SpriteCellMeta {
                palette: Palette::mono([255, 255, 0]).set(0, [0, 0, 0]),
                sprite: 2,
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
            SpriteCellMeta {
                palette: Palette::mono([255, 255, 255]).set(0, [0, 0, 0]),
                sprite: 1,
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
            SpriteCellMeta {
                palette: Palette::mono([255, 0, 255]).set(0, [0, 0, 0]),
                sprite: 72,
            },
            SpriteCellMeta {
                palette: Palette::mono([0, 255, 255]).set(0, [0, 0, 0]),
                sprite: 105,
            },
            SpriteCellMeta {
                palette: Palette::mono([255, 255, 0]).set(0, [0, 0, 0]),
                sprite: 33,
            },
            SpriteCellMeta {
                palette: Palette::mono([0, 255, 0]).set(0, [0, 0, 0]),
                sprite: 19,
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

    assert_that(&actual_image.len()).is_equal_to(&expected_image.len());
    assert_that(&actual_image).is_equal_to(&expected_image);
}

#[test]
fn gray() {
    let mut harness = RenderTestSupportHarness::new(1, 1);

    harness.renderer.update(
        [
            SpriteCellMeta {
                palette: Palette::mono([128, 128, 128]),
                sprite: 0,
            },
        ].iter(),
    );

    // Render the frame.
    harness.renderer.render().unwrap();

    let actual_image = harness.extract_render_result();

    let expected_image = image::load_from_memory(include_bytes!("testdata/50pct_gray.png"))
        .unwrap()
        .to_rgba()
        .into_raw();
    assert_that(&actual_image).is_equal_to(&expected_image);
}
