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

#[cfg(test)]
// The output of the renderer is intended to be pixel-perfect, so the tests are written with
// that in mind.
use image;
use png;

use super::*;
use crate::resources::color::Palette;

/// There's a lot of boilerplate in setting up the offscreen renderer and extracting the rendered
/// image, so we use a separate support fixture to manage that.
struct RenderTestFixture {
    renderer: Renderer,
    sprite_width: u32,
    sprite_height: u32,
    width: u32,
    height: u32,
}

impl RenderTestFixture {
    fn new(width: u32, height: u32) -> RenderTestFixture {
        // Load the test sprite texture.
        // TODO: get rid of this once the sprite-loading code is done.
        let img = include_bytes!("testdata/12x12.png");
        let mut decoder = png::Decoder::new(&img[..]);
        // Need to set this so the index values don't get converted to RGBA.
        decoder.set_transformations(png::Transformations::IDENTITY);
        let mut reader = decoder.read_info().unwrap();
        let mut imgdata = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut imgdata[..]).unwrap();
        let tex = SpriteTexture::new_from_pixels(
            &imgdata[..],
            reader.info().size().0 as usize / 16,
            reader.info().size().1 as usize / 16,
            16,
            16,
        )
        .unwrap();

        let sprite_width = tex.sprite_width();
        let sprite_height = tex.sprite_height();

        let renderer = Renderer::new(
            None,
            (width, height),
            &tex,
            [0, 255, 0].into(),
            wgpu::FilterMode::Nearest,
            wgpu::PresentMode::Fifo,
        )
        .unwrap();

        RenderTestFixture {
            renderer,
            width,
            height,
            sprite_width: sprite_width as u32,
            sprite_height: sprite_height as u32,
        }
    }

    /// Extract the rendered image from the offscreen context.
    fn extract_render_result(&mut self) -> Box<[u8]> {
        self.renderer.fetch_render_output().unwrap()
    }
}

#[test]
fn render_one_cell() {
    let actual_image = {
        let mut fixture = RenderTestFixture::new(1, 1);

        fixture.renderer.update(
            [SpriteCell {
                palette: Palette::mono([255, 255, 255]).set(0, [0, 0, 0]),
                sprite: 1,
                ..Default::default()
            }]
            .iter(),
        );

        // Render the frame.
        fixture.renderer.render_frame().unwrap();

        fixture.extract_render_result()
    };

    let expected_image = image::load_from_memory(include_bytes!("testdata/one_cell.png"))
        .unwrap()
        .to_rgba8()
        .into_raw();
    assert_eq!(&actual_image[..], &expected_image[..]);
}

#[test]
fn render_one_cell_sprite_change() {
    let mut fixture = RenderTestFixture::new(1, 1);

    fixture.renderer.update(
        [SpriteCell {
            palette: Palette::mono([255, 255, 0]).set(0, [0, 0, 0]),
            sprite: 2,
            ..Default::default()
        }]
        .iter(),
    );

    // Render the frame.
    fixture.renderer.render_frame().unwrap();

    let actual_image = fixture.extract_render_result();

    let expected_image = image::load_from_memory(include_bytes!("testdata/one_cell.png"))
        .unwrap()
        .to_rgba8()
        .into_raw();

    // These shouldn't match.
    assert_ne!(&actual_image[..], &expected_image[..]);

    fixture.renderer.update(
        [SpriteCell {
            palette: Palette::mono([255, 255, 255]).set(0, [0, 0, 0]),
            sprite: 1,
            ..Default::default()
        }]
        .iter(),
    );

    // Render the frame.
    fixture.renderer.render_frame().unwrap();

    let actual_image_2 = fixture.extract_render_result();

    // These should.
    assert_eq!(&actual_image_2[..], &expected_image[..]);
}

#[test]
fn render_2x2_with_color() {
    let actual_image = {
        let mut fixture = RenderTestFixture::new(2, 2);

        fixture.renderer.update(
            [
                SpriteCell {
                    palette: Palette::mono([255, 0, 255]).set(0, [0, 0, 0]),
                    sprite: 72,
                    ..Default::default()
                },
                SpriteCell {
                    palette: Palette::mono([0, 255, 255]).set(0, [0, 0, 0]),
                    sprite: 105,
                    ..Default::default()
                },
                SpriteCell {
                    palette: Palette::mono([255, 255, 0]).set(0, [0, 0, 0]),
                    sprite: 33,
                    ..Default::default()
                },
                SpriteCell {
                    palette: Palette::mono([0, 255, 0]).set(0, [0, 0, 0]),
                    sprite: 19,
                    ..Default::default()
                },
            ]
            .iter(),
        );

        // Render the frame.
        fixture.renderer.render_frame().unwrap();

        fixture.extract_render_result()
    };

    let expected_image = image::load_from_memory(include_bytes!("testdata/hi.png"))
        .unwrap()
        .to_rgba8()
        .into_raw();

    assert_eq!(actual_image.len(), expected_image.len());
    assert_eq!(&actual_image[..], &expected_image[..]);
}

#[test]
fn gray() {
    let actual_image = {
        let mut fixture = RenderTestFixture::new(1, 1);

        fixture.renderer.update(
            [SpriteCell {
                palette: Palette::mono([128, 128, 128]),
                sprite: 0,
                ..Default::default()
            }]
            .iter(),
        );

        // Render the frame.
        fixture.renderer.render_frame().unwrap();

        fixture.extract_render_result()
    };

    let expected_image = image::load_from_memory(include_bytes!("testdata/50pct_gray.png"))
        .unwrap()
        .to_rgba8()
        .into_raw();
    assert_eq!(&actual_image[..], &expected_image[..]);
}

#[test]
fn big() {
    let actual_image = {
        let mut fixture = RenderTestFixture::new(680, 10);

        fixture.renderer.update(
            vec![
                SpriteCell {
                    palette: Palette::mono([128, 128, 128]).set(1, [255, 0, 0]),
                    sprite: 1,
                    ..Default::default()
                };
                6800
            ]
            .iter(),
        );

        // Render the frame.
        fixture.renderer.render_frame().unwrap();

        fixture.extract_render_result()
    };

    let expected_image = image::load_from_memory(include_bytes!("testdata/big.png"))
        .unwrap()
        .to_rgba8()
        .into_raw();

    assert_eq!(&actual_image[..], &expected_image[..]);
}

#[test]
fn full_palette() {
    let img = include_bytes!("testdata/full_palette.png");
    let mut decoder = png::Decoder::new(&img[..]);
    // Need to set this so the index values don't get converted to RGBA.
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    let mut imgdata = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut imgdata[..]).unwrap();
    let tex = SpriteTexture::new_from_pixels(
        &imgdata[..],
        reader.info().size().0 as usize,
        reader.info().size().1 as usize,
        1,
        1,
    )
    .unwrap();

    let actual_image = {
        let mut renderer = Renderer::new(
            None,
            (1, 1),
            &tex,
            [0, 255, 0].into(),
            wgpu::FilterMode::Nearest,
            wgpu::PresentMode::Fifo,
        )
        .unwrap();

        renderer.update(
            vec![
                SpriteCell {
                    palette: Default::default(),
                    sprite: 0,
                    ..Default::default()
                };
                1
            ]
            .iter(),
        );

        // Render the frame.
        renderer.render_frame().unwrap();

        renderer.fetch_render_output().unwrap()
    };

    let expected_image =
        image::load_from_memory(include_bytes!("testdata/full_palette_output.png"))
            .unwrap()
            .to_rgba8()
            .into_raw();
    assert_eq!(&actual_image[..], &expected_image[..]);
}
