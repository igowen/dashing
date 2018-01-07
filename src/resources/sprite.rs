use std;
use std::borrow::Borrow;
use std::collections::HashMap;
use itertools::Itertools;

/// Data for an individual sprite.
#[derive(Clone, Debug)]
pub struct Sprite {
    id: usize,
    pixels: Box<[u8]>,
}

/// 8-bit color. Wrapped so color space conversion is easy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color([u8; 3]);

impl Color {
    /// New `Color` from R, G, and B components.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b])
    }

    /// Convert HSV representation to RGB. `h` should be in the range [0, 360], and `s` and `v`
    /// should be in the range [0, 1].
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let vp = (v * 255.0) as u8;
        if h <= 0.0 {
            return Color([vp, vp, vp]);
        }
        let hh = (h % 360.0) / 60.0;
        let i = hh as i32;
        let ff = hh - i as f32;
        let p = ((v * (1.0 - s)) * 255.0) as u8;
        let q = ((v * (1.0 - (s * ff))) * 255.0) as u8;
        let t = ((v * (1.0 - (s * (1.0 - ff)))) * 255.0) as u8;

        match i {
            0 => Color([vp, t, p]),
            1 => Color([q, vp, p]),
            2 => Color([p, vp, t]),
            3 => Color([p, q, vp]),
            4 => Color([t, p, vp]),
            _ => Color([vp, p, q]),
        }
    }
}

/// A 16-color palette.
/// Probably should go in a different module.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    colors: [[u8; 3]; 16],
}

impl Palette {
    /// This lets you set a palette via a builder-style pattern. E.g.,
    ///
    /// ```rust
    /// use dashing::resources::sprite::Palette;
    ///
    /// let p = Palette::default().set(1, [0, 255, 0]);
    ///
    /// assert_eq!(p[1], [0, 255, 0]);
    /// ```
    pub fn set<C: Into<[u8; 3]>>(mut self, i: usize, color: C) -> Self {
        self.colors[i] = color.into();
        self
    }

    /// Create a palette that is all one color. Not particularly useful on its own, but can be
    /// combined with `set()` to generate custom palettes.
    ///
    /// ```
    /// use dashing::resources::sprite::Palette;
    /// let p = Palette::mono([128, 128, 128]);
    ///
    /// for i in 0..16 {
    ///     assert_eq!(p[i], [128, 128, 128]);
    /// }
    /// ```
    pub fn mono<C: Into<[u8; 3]>>(color: C) -> Self {
        Palette { colors: [color.into(); 16] }
    }
}

impl Default for Palette {
    /// Create a palette based on the CGA palette.
    fn default() -> Self {
        Palette {
            colors: [
                [0x00, 0x00, 0x00],
                [0x00, 0x00, 0xaa],
                [0x00, 0xaa, 0x00],
                [0x00, 0xaa, 0xaa],
                [0xaa, 0x00, 0x00],
                [0xaa, 0x00, 0xaa],
                [0xaa, 0x55, 0x00],
                [0xaa, 0xaa, 0xaa],

                [0x55, 0x55, 0x55],
                [0x55, 0x55, 0xff],
                [0x55, 0xff, 0x55],
                [0x55, 0xff, 0xff],
                [0xff, 0x55, 0x55],
                [0xff, 0x55, 0xff],
                [0xff, 0xff, 0x55],
                [0xff, 0xff, 0xff],
            ],
        }
    }
}

impl From<Palette> for [[u8; 3]; 16] {
    fn from(p: Palette) -> Self {
        p.colors
    }
}

impl From<Palette> for [[u8; 4]; 16] {
    fn from(p: Palette) -> Self {
        let mut result = [[0; 4]; 16];
        for (i, o) in p.colors.iter().zip(result.iter_mut()) {
            *o = [i[0], i[1], i[2], 255]
        }
        result
    }
}

impl From<[[u8; 3]; 16]> for Palette {
    fn from(c: [[u8; 3]; 16]) -> Self {
        Palette { colors: c }
    }
}

impl std::ops::Index<usize> for Palette {
    type Output = [u8; 3];
    fn index(&self, i: usize) -> &[u8; 3] {
        self.colors.index(i)
    }
}

impl std::ops::IndexMut<usize> for Palette {
    fn index_mut(&mut self, i: usize) -> &mut [u8; 3] {
        self.colors.index_mut(i)
    }
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
    pub fn new_from_pixels(
        pixels: &[u8],
        width: usize,
        height: usize,
        sprite_width: usize,
        sprite_height: usize,
        sprite_count: usize,
    ) -> Result<SpriteTexture, String> {
        if width % sprite_width != 0 {
            return Err(String::from("Sprite width must divide image width"));
        }
        if height % sprite_height != 0 {
            return Err(String::from("Sprite height must divide image height"));
        }
        if sprite_count > (width / sprite_width) * (height / sprite_height) {
            return Err(String::from(
                "Too many sprites for specified image dimensions",
            ));
        }
        Ok(SpriteTexture {
            width: width,
            height: height,
            sprite_width: sprite_width,
            sprite_height: sprite_height,
            pixels: Box::from(pixels),
            id_map: HashMap::new(), // XXX: fix this
        })
    }
    /// Raw pixels
    pub fn pixels(&'a self) -> &'a [u8] {
        self.pixels.borrow()
    }

    /// sprite id map
    pub fn id_map(&'a self) -> &'a HashMap<usize, usize> {
        &self.id_map
    }
}

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

    /// Convert a `SpriteCollection` to a `SpriteTexture`.
    fn generate_sprite_texture(&self) -> SpriteTexture {
        let sprites_wide = (self.size() as f32).sqrt().ceil() as usize;
        let sprites_high = ((self.size() as f32) / sprites_wide as f32).ceil() as usize;
        let sprite_width = self.dimensions().0;
        let sprite_height = self.dimensions().1;
        let texture_width = sprites_wide * sprite_width as usize;
        let texture_height = sprites_high * sprite_height as usize;
        // It's really unlikely these limits will be exceeded, but I'd rather crap out here than
        // when we try to use the texture in the renderer.
        // With 32x32px sprites, this allows you to have 65k individual sprites.
        assert!(texture_width <= 8192 && texture_height <= 8192);
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

    #[test]
    fn sprite_texture_non_square() {
        let sprites = vec![
            #[cfg_attr(rustfmt, rustfmt_skip)]
            Sprite{
                id: 0,
                pixels: Box::new([1, 0, 1,
                                  0, 1, 0,
                                  0, 1, 0,
                                  1, 0, 1]),
            },
            #[cfg_attr(rustfmt, rustfmt_skip)]
            Sprite{
                id: 1,
                pixels: Box::new([1, 1, 1,
                                  1, 1, 0,
                                  1, 1, 0,
                                  1, 0, 0]),
            },
            #[cfg_attr(rustfmt, rustfmt_skip)]
            Sprite{
                id: 2,
                pixels: Box::new([1, 1, 1,
                                  0, 1, 0,
                                  0, 1, 0,
                                  1, 1, 1]),
            },
        ];
        let collection = TestSpriteCollection {
            sprites: sprites.into_boxed_slice(),
            sprite_width: 3,
            sprite_height: 4,
        };
        let texture = collection.generate_sprite_texture();
        assert_that!(texture.width).is_equal_to(6);
        assert_that!(texture.height).is_equal_to(8);
        assert_that!(texture.sprite_width).is_equal_to(3);
        assert_that!(texture.sprite_height).is_equal_to(4);
        let expected_texture: Vec<u8> = {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            vec![1, 0, 1, 1, 1, 1,
                 0, 1, 0, 1, 1, 0,
                 0, 1, 0, 1, 1, 0,
                 1, 0, 1, 1, 0, 0,
                 1, 1, 1, 0, 0, 0,
                 0, 1, 0, 0, 0, 0,
                 0, 1, 0, 0, 0, 0,
                 1, 1, 1, 0, 0, 0,
            ]
        };
        assert_that!(&texture.pixels).is_equal_to(expected_texture.into_boxed_slice());
    }
}
