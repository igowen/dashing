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
use crate::resources::sprite::SpriteMap;
use std;

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
            width: width,
            height: height,
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
        return &self.data[i];
    }
}

impl std::ops::IndexMut<usize> for SpriteLayer {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        return &mut self.data[i];
    }
}

impl std::ops::Index<(usize, usize)> for SpriteLayer {
    type Output = SpriteCell;
    #[inline]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        return &self.data[y * self.width + x];
    }
}

impl std::ops::IndexMut<(usize, usize)> for SpriteLayer {
    #[inline]
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        return &mut self.data[y * self.width + x];
    }
}

/// Sprites necessary for box drawing routines.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BoxDrawingSprite {
    /// Bottom left corner.
    BottomLeftCorner,
    /// Bottom right corner.
    BottomRightCorner,
    /// Top left corner.
    TopLeftCorner,
    /// Top right corner.
    TopRightCorner,
    /// Horizontal top.
    HorizontalTop,
    /// Horizontal bottom.
    HorizontalBottom,
    /// Vertical left.
    VerticalLeft,
    /// Vertical right.
    VerticalRight,
    /// |-
    TeeRight,
    /// -|
    TeeLeft,
    /// T
    TeeDown,
    /// ⊥
    TeeUp,
    /// Solid fill.
    SolidFill,
}

/// Draw a rectangle with the given sprite set and palette.
/// The same palette is used for each sprite in the output. This isn't a technical requirement, but
/// is more convenient for practical uses.
#[allow(unused)]
pub fn rect<S>(sprite_map: S, width: usize, height: usize, p: Palette) -> SpriteLayer
where
    S: SpriteMap<BoxDrawingSprite>,
{
    let mut out = SpriteLayer::new(width, height);
    out[(0, 0)].sprite = sprite_map.map(BoxDrawingSprite::TopLeftCorner);
    out[(width - 1, 0)].sprite = sprite_map.map(BoxDrawingSprite::TopRightCorner);
    out[(0, height - 1)].sprite = sprite_map.map(BoxDrawingSprite::BottomLeftCorner);
    out[(width - 1, height - 1)].sprite = sprite_map.map(BoxDrawingSprite::BottomRightCorner);

    for i in 1..width - 1 {
        out[(i, 0)].sprite = sprite_map.map(BoxDrawingSprite::HorizontalTop);
        out[(i, height - 1)].sprite = sprite_map.map(BoxDrawingSprite::HorizontalBottom);
    }
    for i in 1..height - 1 {
        out[(0, i)].sprite = sprite_map.map(BoxDrawingSprite::VerticalLeft);
        out[(width - 1, i)].sprite = sprite_map.map(BoxDrawingSprite::VerticalRight);
    }
    for x in 1..width - 1 {
        for y in 1..height - 1 {
            out[(x, y)].sprite = sprite_map.map(BoxDrawingSprite::SolidFill);
        }
    }

    for cell in out.iter_mut() {
        cell.palette = p;
    }

    out
}

/// Draw a message box
pub fn msg_box<S>(
    sprite_map: S,
    width: usize,
    height: usize,
    p: Palette,
    title: &str,
    _message: &str,
) -> SpriteLayer
where
    S: SpriteMap<BoxDrawingSprite> + Copy,
{
    let mut base = rect(sprite_map, width, height, p);
    if height < 4 {
        return base;
    }
    base[(0, 2)].sprite = sprite_map.map(BoxDrawingSprite::TeeRight);
    base[(width - 1, 2)].sprite = sprite_map.map(BoxDrawingSprite::TeeLeft);
    for i in 1..width - 1 {
        base[(i, 2)].sprite = sprite_map.map(BoxDrawingSprite::HorizontalBottom);
    }

    for (i, c) in title.char_indices() {
        if i >= width - 2 {
            break;
        }
        base[(i + 1, 1)].sprite = c as u32;
    }

    if title.len() > width - 2 {
        for i in 0..3 {
            base[(width - 2 - i, 1)].sprite = '.' as u32;
        }
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use hamcrest::*;
    #[test]
    fn stamp_sprite_value() {
        let mut l1 = SpriteLayer::new(4, 4);
        let mut l2 = SpriteLayer::new(2, 3);
        for cell in l2.iter_mut() {
            cell.sprite = 2;
        }
        l2.stamp_onto(&mut l1, 0, 0);

        let expected_sprites: Vec<u32> = {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            vec![2, 2, 0, 0,
                 2, 2, 0, 0,
                 2, 2, 0, 0,
                 0, 0, 0, 0,
            ]
        };

        assert_that!(
            l1.iter().map(|c| c.sprite).collect::<Vec<u32>>(),
            is(equal_to(expected_sprites))
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
        let expected_sprites: Vec<u32> = {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            vec![2, 2, 0, 0,
                 2, 0, 0, 0,
                 2, 2, 0, 0,
                 0, 0, 0, 0,
            ]
        };
        assert_that!(
            l1.iter().map(|c| c.sprite).collect::<Vec<u32>>(),
            is(equal_to(expected_sprites))
        );
    }
}
