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

use std::borrow::Borrow;

/// Trait for mapping symbolic sprites to their position in the sprite texture.
pub trait SpriteMap<E> {
    /// Map the sprite.
    fn map(&self, e: E) -> u32;
}

impl<E, F> SpriteMap<E> for F
where
    F: Fn(E) -> u32,
{
    fn map(&self, e: E) -> u32 {
        self(e)
    }
}

/// Data for an individual sprite.
#[derive(Clone, Debug)]
pub struct Sprite {
    id: usize,
    pixels: Box<[u8]>,
}

/// Output of `SpriteTextureProvider::generate_sprite_texture()`.
pub struct SpriteTexture {
    // Width/height of the texture.
    width: usize,
    height: usize,
    // Width/height of a single sprite.
    sprite_width: usize,
    sprite_height: usize,
    pixels: Box<[u8]>,
}

impl SpriteTexture {
    /// Width
    pub fn width(&self) -> usize {
        self.width
    }
    /// Height
    pub fn height(&self) -> usize {
        self.height
    }
    /// Sprite width
    pub fn sprite_width(&self) -> usize {
        self.sprite_width
    }
    /// Sprite height
    pub fn sprite_height(&self) -> usize {
        self.sprite_height
    }
}

impl<'a> SpriteTexture {
    /// Create a new sprite texture from pixels.
    ///
    /// * `pixels`: Buffer containing the actual pixel data.
    /// * `sprite_width` / `sprite_height`: Sprite width/height in pixels.
    /// * `sprites_per_row`: Number of sprites in each row.
    /// * `rows`: Number of rows.
    pub fn new_from_pixels(
        pixels: &[u8],
        sprite_width: usize,
        sprite_height: usize,
        sprites_per_row: usize,
        rows: usize,
    ) -> Result<SpriteTexture, String> {
        if pixels.len() != (sprite_width * sprites_per_row) * (sprite_height * rows) {
            return Err("Sprite pixel buffer does not match specified parameters".into());
        }
        Ok(SpriteTexture {
            width: sprite_width * sprites_per_row,
            height: sprite_height * rows,
            sprite_width,
            sprite_height,
            pixels: Box::from(pixels),
        })
    }
    /// Raw pixels
    pub fn pixels(&'a self) -> &'a [u8] {
        self.pixels.borrow()
    }
}
