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

use crate::resources::color::Palette;

/// Data for one on-screen sprite instance.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct SpriteCell {
    /// Color for the cell.
    pub palette: Palette,
    /// Sprite index.
    pub sprite: u32,
    /// Transparency.
    pub transparent: bool,
}

/// A 2D array of sprite cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteLayer {
    width: usize,
    height: usize,
    data: Box<[SpriteCell]>,
}

impl SpriteLayer {
    /// Create a new `SpriteLayer` with the given width and height.
    pub fn new(width: usize, height: usize) -> Self {
        SpriteLayer {
            width,
            height,
            data: vec![SpriteCell::default(); width * height].into_boxed_slice(),
        }
    }

    /// Get an iterator over all of the cells in the layer.
    pub fn iter(&self) -> std::slice::Iter<SpriteCell> {
        self.data.iter()
    }

    /// Get a mutable iterator over all of the cells in the layer.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<SpriteCell> {
        self.data.iter_mut()
    }

    /// Copy the entirety of this layer onto the specified layer.
    pub fn stamp_onto(&self, other: &mut SpriteLayer, offset_x: usize, offset_y: usize) {
        // +------------------+
        // |                  |
        // |  (o_x, o_y)      |
        // |      +-------------------+
        // |      |XXXXXXXXXXX|       |
        // |      |XXXXXXXXXXX|       |
        // +------|-----------+       |
        //        |                   |
        //        +-------------------+
        //
        // +------------+
        // |            |
        // |    +------+|
        // |    |XXXXXX||
        // |    |XXXXXX||
        // |    +------+|
        // +------------+
        let truncated_width = self.width.min(other.width.saturating_sub(offset_x));
        let truncated_height = self.height.min(other.height.saturating_sub(offset_y));
        for x in 0..truncated_width {
            for y in 0..truncated_height {
                if !self[(x, y)].transparent {
                    other[(offset_x + x, offset_y + y)] = self[(x, y)];
                }
            }
        }
    }

    /// Clear (set to 0) all the sprites in the layer. Does not affect colors.
    pub fn clear_sprites(&mut self) {
        for c in self.iter_mut() {
            c.sprite = 0;
        }
    }

    /// Clear sprites and colors.
    pub fn clear(&mut self) {
        for c in self.iter_mut() {
            *c = SpriteCell::default();
        }
    }

    /// Get width of the layer.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Get height of the layer.
    pub fn height(&self) -> usize {
        self.height
    }
}

impl std::ops::Index<usize> for SpriteLayer {
    type Output = SpriteCell;
    #[inline]
    fn index(&self, i: usize) -> &Self::Output {
        &self.data[i]
    }
}

impl std::ops::IndexMut<usize> for SpriteLayer {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.data[i]
    }
}

impl std::ops::Index<(usize, usize)> for SpriteLayer {
    type Output = SpriteCell;
    #[inline]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[y * self.width + x]
    }
}

impl std::ops::IndexMut<(usize, usize)> for SpriteLayer {
    #[inline]
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.data[y * self.width + x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stamp_sprite_value() {
        let mut l1 = SpriteLayer::new(4, 4);
        let mut l2 = SpriteLayer::new(2, 3);
        for cell in l2.iter_mut() {
            cell.sprite = 2;
        }
        l2.stamp_onto(&mut l1, 0, 0);

        #[rustfmt::skip]
        let expected_sprites: Vec<u32> = {
            vec![2, 2, 0, 0,
                 2, 2, 0, 0,
                 2, 2, 0, 0,
                 0, 0, 0, 0,
            ]
        };

        assert_eq!(
            l1.iter().map(|c| c.sprite).collect::<Vec<u32>>(),
            expected_sprites
        );
    }
    #[test]
    fn stamp_transparency() {
        let mut l1 = SpriteLayer::new(4, 4);
        let mut l2 = SpriteLayer::new(2, 3);
        for cell in l2.iter_mut() {
            cell.sprite = 2;
        }
        l2[(1, 1)].transparent = true;
        l2.stamp_onto(&mut l1, 0, 0);

        #[rustfmt::skip]
        let expected_sprites: Vec<u32> = {
            vec![2, 2, 0, 0,
                 2, 0, 0, 0,
                 2, 2, 0, 0,
                 0, 0, 0, 0,
            ]
        };
        assert_eq!(
            l1.iter().map(|c| c.sprite).collect::<Vec<u32>>(),
            expected_sprites
        );
    }
}
