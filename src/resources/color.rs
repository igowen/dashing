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

use std;

/// 8-bit RGB color. Wrapped so color space conversion is easy.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color([u8; 3]);

impl Color {
    /// New `Color` from R, G, and B components.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b])
    }

    /// Convert HSV (hue, saturation, value) representation to RGB. `h` should be in the range [0,
    /// 360], and `s` and `v` should be in the range [0, 1]; however, this function will still work
    /// for values outside those ranges (`h` is interpreted mod 360, and `s` and `v` are clamped).
    ///
    /// ```
    /// # use dashing::resources::color::Color;
    /// assert_eq!(
    ///     Color::from_hsv(0.0, 1.0, 1.0),
    ///     Color::new(255, 0, 0),
    ///     "HSV(0.0, 1.0, 1.0) == RGB(255, 0, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(0.0, 1.0, 0.5),
    ///     Color::new(127, 0, 0),
    ///     "HSV(0.0, 1.0, 0.5) == RGB(127, 0, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(60.0, 1.0, 1.0),
    ///     Color::new(255, 255, 0),
    ///     "HSV(60.0, 1.0, 1.0) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(53.0, 0.0, 1.0),
    ///     Color::new(255, 255, 255),
    ///     "HSV(53.0, 0.0, 1.0) == RGB(255, 255, 255)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(21.0, 0.0, 0.0),
    ///     Color::new(0, 0, 0),
    ///     "HSV(21.0, 0.0 0.0) == RGB(0, 0, 0)"
    /// );
    /// // Out of bounds hue
    /// assert_eq!(
    ///     Color::from_hsv(420.0, 1.0, 1.0),
    ///     Color::new(255, 255, 0),
    ///     "HSV(420.0, 1.0, 1.0) == HSV(60.0, 1.0, 1.0) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(-300.0, 1.0, 1.0),
    ///     Color::new(255, 255, 0),
    ///     "HSV(-300.0, 1.0, 1.0) == HSV(60.0, 1.0, 1.0) == RGB(255, 255, 0)"
    /// );
    /// // Out of bounds saturation
    /// assert_eq!(
    ///     Color::from_hsv(60.0, 200.0, 1.0),
    ///     Color::new(255, 255, 0),
    ///     "HSV(60.0, 200.0, 1.0) == HSV(60.0, 1.0, 1.0) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(60.0, -200.0, 1.0),
    ///     Color::new(255, 255, 255),
    ///     "HSV(60.0, -200.0, 1.0) == HSV(60.0, 0.0, 1.0) == RGB(255, 255, 255)"
    /// );
    /// // Out of bounds value
    /// assert_eq!(
    ///     Color::from_hsv(60.0, 1.0, 200.0),
    ///     Color::new(255, 255, 0),
    ///     "HSV(60.0, 1.0, 200.0) == HSV(60.0, 1.0, 1.0) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsv(60.0, 1.0, -200.0),
    ///     Color::new(0, 0, 0),
    ///     "HSV(60.0, 1.0, -200.0) == HSV(60.0, 1.0, 0.0) == RGB(0, 0, 0)"
    /// );
    ///
    /// ```
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let hh = ((h % 360.0) + if h < 0.0 { 360.0 } else { 0.0 }) / 60.0;
        let ss = s.min(1.0).max(0.0);
        let vv = v.min(1.0).max(0.0);

        let chroma = vv * ss;
        let x = chroma * (1.0 - (hh % 2.0 - 1.0).abs());

        let m = vv - chroma;

        let i = ((chroma + m) * 255.0) as u8;
        let j = ((x + m) * 255.0) as u8;
        let k = (m * 255.0) as u8;

        match hh as i32 {
            0 => Color([i, j, k]),
            1 => Color([j, i, k]),
            2 => Color([k, i, j]),
            3 => Color([k, j, i]),
            4 => Color([j, k, i]),
            _ => Color([i, k, j]),
        }
    }

    /// Convert HSL (hue, saturation, lightness) representation to RGB. `h` should be in the range
    /// [0, 360], and `s` and `v` should be in the range [0, 1]; however, this function will still
    /// work for values outside those ranges (`h` is interpreted mod 360, and `s` and `v` are
    /// clamped).
    ///
    /// ```
    /// # use dashing::resources::color::Color;
    /// assert_eq!(
    ///     Color::from_hsl(0.0, 1.0, 0.5),
    ///     Color::new(255, 0, 0),
    ///     "HSL(0.0, 1.0, 0.0) == RGB(255, 0, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(0.0, 1.0, 0.25),
    ///     Color::new(127, 0, 0),
    ///     "HSL(0.0, 1.0, 0.25) == RGB(127, 0, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(60.0, 1.0, 0.5),
    ///     Color::new(255, 255, 0),
    ///     "HSL(60.0, 1.0, 0.5) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(53.0, 0.0, 1.0),
    ///     Color::new(255, 255, 255),
    ///     "HSL(53.0, 0.0, 1.0) == RGB(255, 255, 255)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(53.0, 1.0, 1.0),
    ///     Color::new(255, 255, 255),
    ///     "HSL(53.0, 1.0, 1.0) == RGB(255, 255, 255)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(21.0, 0.0, 0.0),
    ///     Color::new(0, 0, 0),
    ///     "HSL(21.0, 0.0, 0.0) == RGB(0, 0, 0)"
    /// );
    /// // Out of bounds hue
    /// assert_eq!(
    ///     Color::from_hsl(420.0, 1.0, 0.5),
    ///     Color::new(255, 255, 0),
    ///     "HSL(420.0, 1.0, 0.5) == HSL(60.0, 1.0, 0.5) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(-300.0, 1.0, 0.5),
    ///     Color::new(255, 255, 0),
    ///     "HSL(-300.0, 1.0, 0.5) == HSL(60.0, 1.0, 0.5) == RGB(255, 255,0)"
    /// );
    /// // Out of bounds saturation
    /// assert_eq!(
    ///     Color::from_hsl(60.0, 200.0, 0.5),
    ///     Color::new(255, 255, 0),
    ///     "HSL(60.0, 200.0, 0.5) == HSL(60.0, 1.0, 0.5) == RGB(255, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(60.0, -200.0, 0.5),
    ///     Color::new(127, 127, 127),
    ///     "HSL(60.0, -200.0, 0.5) == HSL(60.0, 0.0, 0.5) == RGB(127, 127, 127)"
    /// );
    /// // Out of bounds lightness
    /// assert_eq!(
    ///     Color::from_hsl(60.0, 1.0, 200.0),
    ///     Color::new(255, 255, 255),
    ///     "HSL(60.0, 1.0, 200.0) == HSL(60.0, 1.0, 1.0) == RGB(255, 255, 255)"
    /// );
    /// assert_eq!(
    ///     Color::from_hsl(60.0, 1.0, -200.0),
    ///     Color::new(0, 0, 0),
    ///     "HSL(60.0, 1.0, 0.0) == HSL(60.0, 1.0, 0.0) == RGB(0, 0, 0)"
    /// );
    ///
    /// ```
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let hh = ((h % 360.0) + if h < 0.0 { 360.0 } else { 0.0 }) / 60.0;
        let ss = s.min(1.0).max(0.0);
        let ll = l.min(1.0).max(0.0);

        let chroma = (1.0 - (2.0 * ll - 1.0).abs()) * ss;
        let x = chroma * (1.0 - (hh % 2.0 - 1.0).abs());

        let m = ll - chroma / 2.0;

        let i = ((chroma + m) * 255.0) as u8;
        let j = ((x + m) * 255.0) as u8;
        let k = (m * 255.0) as u8;

        match hh as i32 {
            0 => Color([i, j, k]),
            1 => Color([j, i, k]),
            2 => Color([k, i, j]),
            3 => Color([k, j, i]),
            4 => Color([j, k, i]),
            _ => Color([i, k, j]),
        }
    }

    /// Convert to HSL color
    /// ```
    /// # use dashing::resources::color::Color;
    /// assert_eq!(Color::from_hsv(0.0, 1.0, 1.0).to_hsv(), [0.0, 1.0, 1.0]);
    /// assert_eq!(Color::from_hsv(60.0, 1.0, 1.0).to_hsv(), [60.0, 1.0, 1.0]);
    /// ```
    pub fn to_hsv(&self) -> [f32; 3] {
        if self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 {
            return [0.0, 0.0, 0.0];
        }
        let rf = self.0[0] as f32 / 255.0;
        let gf = self.0[1] as f32 / 255.0;
        let bf = self.0[2] as f32 / 255.0;
        let min = rf.min(gf.min(bf));
        let max = rf.max(gf.max(bf));
        let v = max;
        let s = (max - min) / max;
        let h;
        if max - min == 0.0 {
            h = 0.0;
        } else if rf == max {
            h = (gf - bf) / (max - min);
        } else if gf == max {
            h = 2.0 + (bf - rf) / (max - min);
        } else {
            // bf == max
            h = 4.0 + (rf - gf) / (max - min);
        }
        [(h * 60.0) % 360.0, s, v]
    }

    /// Convert HWB (hue, white, black) representation to RGB.
    ///
    /// ```
    /// # use dashing::resources::color::Color;
    /// assert_eq!(
    ///     Color::from_hwb(120.0, 0.0, 0.0),
    ///     Color::new(0, 255, 0),
    ///     "HWB(120.0, 0.0, 0.0) == RGB(0, 255, 0)"
    /// );
    /// assert_eq!(
    ///     Color::from_hwb(120.0, 0.5, 0.5),
    ///     Color::new(127, 127, 127),
    ///     "HWB(120.0, 0.5, 0.5) == RGB(127, 127, 127)"
    /// );
    /// assert_eq!(
    ///     Color::from_hwb(120.0, 0.5, 0.0),
    ///     Color::new(127, 255, 127),
    ///     "HWB(120.0, 0.5, 0.0) == RGB(127, 255, 127)"
    /// );
    /// assert_eq!(
    ///     Color::from_hwb(120.0, 0.0, 0.5),
    ///     Color::new(0, 127, 0),
    ///     "HWB(120.0, 0.0, 0.5) == RGB(0, 127, 0)"
    /// );
    /// // W/B values can be > 1.0 but it mashes the result into an even gray.
    /// assert_eq!(
    ///     Color::from_hwb(120.0, 25.0, 75.0),
    ///     Color::new(63, 63, 63),
    ///     "HWB(120.0, 25.0, 75.0) == RGB(63, 63, 63)"
    /// );
    /// ```
    pub fn from_hwb(h: f32, w: f32, b: f32) -> Self {
        let ww = w.max(0.0);
        let bb = b.max(0.0);
        let www = if ww + bb > 1.0 { ww / (ww + bb) } else { ww };
        let bbb = if ww + bb > 1.0 { bb / (ww + bb) } else { bb };
        Self::from_hsv(h, 1.0 - www / (1.0 - bbb), 1.0 - bbb)
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [
            c.0[0] as f32 / 255.0,
            c.0[1] as f32 / 255.0,
            c.0[2] as f32 / 255.0,
            1.0,
        ]
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(c: Color) -> Self {
        (
            c.0[0] as f32 / 255.0,
            c.0[1] as f32 / 255.0,
            c.0[2] as f32 / 255.0,
        )
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Color([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8])
    }
}

impl From<Color> for [u8; 3] {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<[u8; 3]> for Color {
    fn from(c: [u8; 3]) -> Self {
        Color([c[0], c[1], c[2]])
    }
}

impl Default for Color {
    fn default() -> Self {
        [0, 0, 0].into()
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
    /// ```
    /// use dashing::resources::color::Palette;
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
    /// use dashing::resources::color::Palette;
    /// let p = Palette::mono([128, 128, 128]);
    ///
    /// for i in 0..16 {
    ///     assert_eq!(p[i], [128, 128, 128]);
    /// }
    /// ```
    pub fn mono<C: Into<[u8; 3]>>(color: C) -> Self {
        Palette {
            colors: [color.into(); 16],
        }
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

/// Enum representing palette index. Using this enum instead of `usize` means we can theoretically
/// avoid some amount of runtime bounds checking.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PaletteIndex {
    /// 0
    P0,
    /// 1
    P1,
    /// 2
    P2,
    /// 3
    P3,
    /// 4
    P4,
    /// 5
    P5,
    /// 6
    P6,
    /// 7
    P7,
    /// 8
    P8,
    /// 9
    P9,
    /// 10
    P10,
    /// 11
    P11,
    /// 12
    P12,
    /// 13
    P13,
    /// 14
    P14,
    /// 15
    P15,
}

impl PaletteIndex {
    /// Iterator over every PaletteIndex value in order.
    pub fn iter() -> impl Iterator<Item = &'static PaletteIndex> {
        static INDICES: [PaletteIndex; 16] = [
            PaletteIndex::P0,
            PaletteIndex::P1,
            PaletteIndex::P2,
            PaletteIndex::P3,
            PaletteIndex::P4,
            PaletteIndex::P5,
            PaletteIndex::P6,
            PaletteIndex::P7,
            PaletteIndex::P8,
            PaletteIndex::P9,
            PaletteIndex::P10,
            PaletteIndex::P11,
            PaletteIndex::P12,
            PaletteIndex::P13,
            PaletteIndex::P14,
            PaletteIndex::P15,
        ];
        INDICES.iter()
    }
}

impl From<PaletteIndex> for u8 {
    fn from(p: PaletteIndex) -> u8 {
        match p {
            PaletteIndex::P0 => 0,
            PaletteIndex::P1 => 1,
            PaletteIndex::P2 => 2,
            PaletteIndex::P3 => 3,
            PaletteIndex::P4 => 4,
            PaletteIndex::P5 => 5,
            PaletteIndex::P6 => 6,
            PaletteIndex::P7 => 7,
            PaletteIndex::P8 => 8,
            PaletteIndex::P9 => 9,
            PaletteIndex::P10 => 10,
            PaletteIndex::P11 => 11,
            PaletteIndex::P12 => 12,
            PaletteIndex::P13 => 13,
            PaletteIndex::P14 => 14,
            PaletteIndex::P15 => 15,
        }
    }
}

impl From<PaletteIndex> for usize {
    fn from(p: PaletteIndex) -> usize {
        <PaletteIndex as Into<u8>>::into(p) as usize
    }
}

impl std::ops::Index<PaletteIndex> for Palette {
    type Output = [u8; 3];
    fn index(&self, i: PaletteIndex) -> &Self::Output {
        match i {
            PaletteIndex::P0 => &self.colors[0],
            PaletteIndex::P1 => &self.colors[1],
            PaletteIndex::P2 => &self.colors[2],
            PaletteIndex::P3 => &self.colors[3],
            PaletteIndex::P4 => &self.colors[4],
            PaletteIndex::P5 => &self.colors[5],
            PaletteIndex::P6 => &self.colors[6],
            PaletteIndex::P7 => &self.colors[7],
            PaletteIndex::P8 => &self.colors[8],
            PaletteIndex::P9 => &self.colors[9],
            PaletteIndex::P10 => &self.colors[10],
            PaletteIndex::P11 => &self.colors[11],
            PaletteIndex::P12 => &self.colors[12],
            PaletteIndex::P13 => &self.colors[13],
            PaletteIndex::P14 => &self.colors[14],
            PaletteIndex::P15 => &self.colors[15],
        }
    }
}

impl std::ops::IndexMut<PaletteIndex> for Palette {
    fn index_mut(&mut self, i: PaletteIndex) -> &mut Self::Output {
        match i {
            PaletteIndex::P0 => &mut self.colors[0],
            PaletteIndex::P1 => &mut self.colors[1],
            PaletteIndex::P2 => &mut self.colors[2],
            PaletteIndex::P3 => &mut self.colors[3],
            PaletteIndex::P4 => &mut self.colors[4],
            PaletteIndex::P5 => &mut self.colors[5],
            PaletteIndex::P6 => &mut self.colors[6],
            PaletteIndex::P7 => &mut self.colors[7],
            PaletteIndex::P8 => &mut self.colors[8],
            PaletteIndex::P9 => &mut self.colors[9],
            PaletteIndex::P10 => &mut self.colors[10],
            PaletteIndex::P11 => &mut self.colors[11],
            PaletteIndex::P12 => &mut self.colors[12],
            PaletteIndex::P13 => &mut self.colors[13],
            PaletteIndex::P14 => &mut self.colors[14],
            PaletteIndex::P15 => &mut self.colors[15],
        }
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

/// Procedural palette
#[derive(Default)]
pub struct ProceduralPalette([ColorExpression; 16]);

impl ProceduralPalette {
    /// Evaluate this palette.
    ///
    /// ```
    /// # use dashing::resources::color::*;
    /// let p = Palette::default();
    /// assert_eq!(p, ProceduralPalette::default().eval(p));
    /// ```
    pub fn eval(&self, p: Palette) -> Palette {
        fn convert((r, g, b): (f32, f32, f32)) -> [u8; 3] {
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
        }
        Palette {
            colors: [
                convert(self.0[0].eval(p, PaletteIndex::P0)),
                convert(self.0[1].eval(p, PaletteIndex::P1)),
                convert(self.0[2].eval(p, PaletteIndex::P2)),
                convert(self.0[3].eval(p, PaletteIndex::P3)),
                convert(self.0[4].eval(p, PaletteIndex::P4)),
                convert(self.0[5].eval(p, PaletteIndex::P5)),
                convert(self.0[6].eval(p, PaletteIndex::P6)),
                convert(self.0[7].eval(p, PaletteIndex::P7)),
                convert(self.0[8].eval(p, PaletteIndex::P8)),
                convert(self.0[9].eval(p, PaletteIndex::P9)),
                convert(self.0[10].eval(p, PaletteIndex::P10)),
                convert(self.0[11].eval(p, PaletteIndex::P11)),
                convert(self.0[12].eval(p, PaletteIndex::P12)),
                convert(self.0[13].eval(p, PaletteIndex::P13)),
                convert(self.0[14].eval(p, PaletteIndex::P14)),
                convert(self.0[15].eval(p, PaletteIndex::P15)),
            ],
        }
    }
}

/// ColorExpression is a way to represent a conversion from one color to another.
#[derive(Debug)]
pub enum ColorExpression {
    /// Null transform: do nothing and just copy the underlying palette at the same index.
    Null,
    /// Straight copy of a different palette index.
    PaletteMap(PaletteIndex),
    /// RGB construction
    Rgb(
        Box<ValueExpression>,
        Box<ValueExpression>,
        Box<ValueExpression>,
    ),
    /// Use the same value for all 3 channels.
    Mono(Box<ValueExpression>),
}

impl ColorExpression {
    /// Evaluate this expression, producing an `(f32, f32, f32)` tuple.
    fn eval(&self, p: Palette, i: PaletteIndex) -> (f32, f32, f32) {
        match self {
            ColorExpression::Null => Color::from(p[i]).into(),
            ColorExpression::PaletteMap(pi) => Color::from(p[*pi]).into(),
            ColorExpression::Rgb(r, g, b) => (r.eval(p, i), g.eval(p, i), b.eval(p, i)),
            ColorExpression::Mono(v) => {
                let vv = v.eval(p, i);
                (vv, vv, vv)
            }
        }
    }
}

impl Default for ColorExpression {
    fn default() -> Self {
        ColorExpression::Null
    }
}

/// Expression representing a single channel.
///
/// ```
/// # use dashing::resources::color::*;
/// let p = Palette::default();
/// assert_eq!(ValueExpression::Constant(42.0).eval(p, PaletteIndex::P0), 42.0);
/// assert_eq!(
///     ValueExpression::Add(Box::new(ValueExpression::Constant(12.0)),
///                          Box::new(ValueExpression::Mul(Box::new(ValueExpression::Constant(2.0)),
///                                                        Box::new(ValueExpression::Constant(3.0)))))
///         .eval(p, PaletteIndex::P0),
///     18.0);
/// assert_eq!(
///     ValueExpression::Red(Box::new(ColorExpression::PaletteMap(PaletteIndex::P15))).eval(p, PaletteIndex::P0),
///     1.0);
/// assert_eq!(
///     ValueExpression::Neg(Box::new(ValueExpression::Constant(42.0))).eval(p, PaletteIndex::P0),
///     -42.0);
/// ```
#[derive(Debug)]
pub enum ValueExpression {
    /// Constant value.
    Constant(f32),
    /// Extract the red channel of a color.
    Red(Box<ColorExpression>),
    /// Extract the green channel of a color.
    Green(Box<ColorExpression>),
    /// Extract the blue channel of a color.
    Blue(Box<ColorExpression>),
    /// Negate a value.
    Neg(Box<ValueExpression>),
    /// Add two values.
    Add(Box<ValueExpression>, Box<ValueExpression>),
    /// Subtract two values.
    Sub(Box<ValueExpression>, Box<ValueExpression>),
    /// Multiply two values.
    Mul(Box<ValueExpression>, Box<ValueExpression>),
    /// Divide two values.
    Div(Box<ValueExpression>, Box<ValueExpression>),
    /// Return the minimum of the two values.
    Min(Box<ValueExpression>, Box<ValueExpression>),
    /// Return the maximum of the two values.
    Max(Box<ValueExpression>, Box<ValueExpression>),
}

impl ValueExpression {
    /// Evaluate this expression, producing an `f32` value.
    pub fn eval(&self, p: Palette, i: PaletteIndex) -> f32 {
        match self {
            ValueExpression::Constant(f) => *f,
            ValueExpression::Red(c) => c.eval(p, i).0,
            ValueExpression::Green(c) => c.eval(p, i).1,
            ValueExpression::Blue(c) => c.eval(p, i).2,
            ValueExpression::Neg(v) => -v.eval(p, i),
            ValueExpression::Add(a, b) => a.eval(p, i) + b.eval(p, i),
            ValueExpression::Sub(a, b) => a.eval(p, i) - b.eval(p, i),
            ValueExpression::Mul(a, b) => a.eval(p, i) * b.eval(p, i),
            ValueExpression::Div(a, b) => a.eval(p, i) / b.eval(p, i),
            ValueExpression::Min(a, b) => f32::min(a.eval(p, i), b.eval(p, i)),
            ValueExpression::Max(a, b) => f32::max(a.eval(p, i), b.eval(p, i)),
        }
    }
}
