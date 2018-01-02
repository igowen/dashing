use std::collections::HashMap;
use itertools::Itertools;

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
  id_map: HashMap<usize, usize>,
}

/// A 16-color palette.
pub type Palette = [[u8; 3]; 16];

/// `SpriteCollection` is an interface that the library uses to assemble a single sprite texture
/// out of multiple sprite definitions.
/// Sprites are stored as arrays of `u8`s, with each u8 representing one pixel; the value of the u8
/// is an index into a 16-color palette.
pub trait SpriteCollection {
  /// Iterator over all of the sprites in this collection.
  type Iter: Iterator<Item = Sprite>;
  /// Sprite size, in pixels. Must be uniform across the collection.
  fn dimensions(&self) -> (u32, u32);
  /// Number of sprites in the collection.
  fn size(&self) -> usize;
  /// Iterator over every sprite in the collection.
  fn iter(&self) -> Self::Iter;
  /// Retrive a particular sprite.
  fn get(&self, sprite: usize) -> Option<Sprite>;
}

/// `SpriteTextureProvider` is an interface for acquiring a single-texture representation of
/// multiple sprites.
pub trait SpriteTextureProvider {
  /// Generate the texture.
  fn generate_sprite_texture(&self) -> SpriteTexture;
}

/// Generic implementation for `SpriteCollection`.
impl<T: SpriteCollection> SpriteTextureProvider for T {
  fn generate_sprite_texture(&self) -> SpriteTexture {
    let sprites_wide = (self.size() as f32).sqrt().ceil() as usize;
    let sprites_high = ((self.size() as f32) / sprites_wide as f32).ceil() as usize;
    let sprite_width = self.dimensions().0;
    let sprite_height = self.dimensions().1;
    let texture_width = sprites_wide * sprite_width as usize;
    let texture_height = sprites_high * sprite_height as usize;
    let mut pixels = Vec::<u8>::with_capacity(texture_width * texture_height);
    let mut id_map = HashMap::<usize, usize>::with_capacity(self.size());
    let mut i = 0;
    for chunk in &self.iter().chunks(sprites_wide) {
      let sprite_row: Vec<Sprite> = chunk.collect();
      for s in sprite_row.iter() {
        id_map.insert(s.id, i);
        i += 1;
      }
      for y in 0..sprite_height {
        for s in sprite_row.iter() {
          for x in 0..sprite_width {
            pixels.push(s.pixels[(y * sprite_width + x) as usize]);
          }
        }
        if sprite_row.len() < sprites_wide {
          pixels.extend(vec![
            0;
            (sprites_wide - sprite_row.len()) *
                sprite_width as usize
          ]);
        }
      }
    }
    SpriteTexture {
      width: texture_width,
      height: texture_height,
      sprite_width: self.dimensions().0 as usize,
      sprite_height: self.dimensions().1 as usize,
      pixels: pixels.into_boxed_slice(),
      id_map: id_map,
    }
  }
}

#[cfg(test)]
mod tests {
  use spectral::prelude::*;
  use std;
  use super::*;

  struct TestSpriteCollection {
    sprites: Box<[Sprite]>,
    sprite_width: usize,
    sprite_height: usize,
  }

  impl SpriteCollection for TestSpriteCollection {
    type Iter = std::vec::IntoIter<Sprite>;
    fn dimensions(&self) -> (u32, u32) {
      (self.sprite_width as u32, self.sprite_height as u32)
    }
    fn size(&self) -> usize {
      self.sprites.len()
    }
    fn get(&self, _sprite: usize) -> Option<Sprite> {
      unimplemented!();
    }
    fn iter(&self) -> Self::Iter {
      let v: Vec<Sprite> = self.sprites.iter().cloned().collect();
      return v.into_iter();
    }
  }

  #[test]
  fn sprite_texture_basic() {
    let sprites = vec![
      #[cfg_attr(rustfmt, rustfmt_skip)]
      Sprite{
        id: 0,
        pixels: Box::new([0, 1, 1, 0,
                          1, 0, 0, 1,
                          0, 0, 1, 1,
                          0, 0, 0, 1]),
      },
      #[cfg_attr(rustfmt, rustfmt_skip)]
      Sprite{
        id: 1,
        pixels: Box::new([1, 1, 1, 1,
                          1, 1, 1, 0,
                          1, 1, 0, 0,
                          1, 0, 0, 0]),
      },
    ];
    let collection = TestSpriteCollection {
      sprites: sprites.into_boxed_slice(),
      sprite_width: 4,
      sprite_height: 4,
    };
    let texture = collection.generate_sprite_texture();
    assert_that!(&texture.width).is_equal_to(8);
    assert_that!(&texture.height).is_equal_to(4);
    assert_that!(&texture.sprite_width).is_equal_to(4);
    assert_that!(&texture.sprite_height).is_equal_to(4);
    let expected_texture: Vec<u8> = {
      #[cfg_attr(rustfmt, rustfmt_skip)]
      vec![0, 1, 1, 0, 1, 1, 1, 1,
           1, 0, 0, 1, 1, 1, 1, 0,
           0, 0, 1, 1, 1, 1, 0, 0,
           0, 0, 0, 1, 1, 0, 0, 0,
      ]
    };
    assert_that!(&texture.pixels).is_equal_to(expected_texture.into_boxed_slice());
  }

  #[test]
  fn sprite_texture_uneven() {
    let sprites = vec![
      #[cfg_attr(rustfmt, rustfmt_skip)]
      Sprite{
        id: 0,
        pixels: Box::new([0, 0, 0, 0,
                          0, 0, 0, 1,
                          0, 0, 1, 1,
                          0, 1, 1, 1]),
      },
      #[cfg_attr(rustfmt, rustfmt_skip)]
      Sprite{
        id: 1,
        pixels: Box::new([1, 1, 1, 1,
                          1, 1, 1, 0,
                          1, 1, 0, 0,
                          1, 0, 0, 0]),
      },
      #[cfg_attr(rustfmt, rustfmt_skip)]
      Sprite{
        id: 2,
        pixels: Box::new([1, 1, 1, 1,
                          0, 1, 1, 0,
                          0, 1, 1, 0,
                          1, 1, 1, 1]),
      },
    ];
    let collection = TestSpriteCollection {
      sprites: sprites.into_boxed_slice(),
      sprite_width: 4,
      sprite_height: 4,
    };
    let texture = collection.generate_sprite_texture();
    assert_that!(texture.width).is_equal_to(8);
    assert_that!(texture.height).is_equal_to(8);
    assert_that!(texture.sprite_width).is_equal_to(4);
    assert_that!(texture.sprite_height).is_equal_to(4);
    let expected_texture: Vec<u8> = {
      #[cfg_attr(rustfmt, rustfmt_skip)]
      vec![0, 0, 0, 0, 1, 1, 1, 1,
           0, 0, 0, 1, 1, 1, 1, 0,
           0, 0, 1, 1, 1, 1, 0, 0,
           0, 1, 1, 1, 1, 0, 0, 0,
           1, 1, 1, 1, 0, 0, 0, 0,
           0, 1, 1, 0, 0, 0, 0, 0,
           0, 1, 1, 0, 0, 0, 0, 0,
           1, 1, 1, 1, 0, 0, 0, 0,
      ]
    };
    assert_that!(&texture.pixels).is_equal_to(expected_texture.into_boxed_slice());
  }
}
