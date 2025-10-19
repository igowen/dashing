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

/// 8-bit RGB color. Wrapped so color space conversion is easy.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color([u8; 3]);

impl Color {
    /// New `Color` from R, G, and B components.
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color([red, green, blue])
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
    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let hh = ((hue % 360.0) + if hue < 0.0 { 360.0 } else { 0.0 }) / 60.0;
        let ss = saturation.min(1.0).max(0.0);
        let vv = value.min(1.0).max(0.0);

        let chroma = vv * ss;
        let secondary = chroma * (1.0 - (hh % 2.0 - 1.0).abs());

        let m = vv - chroma;

        let i = ((chroma + m) * 255.0) as u8;
        let j = ((secondary + m) * 255.0) as u8;
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
    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Self {
        let hh = ((hue % 360.0) + if hue < 0.0 { 360.0 } else { 0.0 }) / 60.0;
        let ss = saturation.min(1.0).max(0.0);
        let ll = lightness.min(1.0).max(0.0);

        let chroma = (1.0 - (2.0 * ll - 1.0).abs()) * ss;
        let secondary = chroma * (1.0 - (hh % 2.0 - 1.0).abs());

        let m = ll - chroma / 2.0;

        let i = ((chroma + m) * 255.0) as u8;
        let j = ((secondary + m) * 255.0) as u8;
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
        } else if (rf - max).abs() < f32::EPSILON {
            h = (gf - bf) / (max - min);
        } else if (gf - max).abs() < f32::EPSILON {
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

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
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

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color([r, g, b])
    }
}

impl From<Color> for wgpu::Color {
    fn from(c: Color) -> Self {
        wgpu::Color {
            r: c.0[0] as f64 / 255.0,
            g: c.0[1] as f64 / 255.0,
            b: c.0[2] as f64 / 255.0,
            a: 1.0,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        [0, 0, 0].into()
    }
}

/// A 16-color palette.
/// Probably should go in a different module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    colors: Box<[[u8; 3]]>,
}

impl Palette {
    /// Create a new palette with explicitly specified colors.
    pub fn new<T>(colors: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<[u8; 3]>,
    {
        Self {
            colors: colors.into_iter().map(|c| c.into()).collect(),
        }
    }

    /// This lets you set a palette via a builder-style pattern. E.g.,
    ///
    /// ```
    /// use dashing::resources::color::Palette;
    ///
    /// let p = Palette::default().set(1, [0, 255, 0]);
    ///
    /// assert_eq!(p[1], [0, 255, 0]);
    /// ```
    #[must_use]
    pub fn set<C: Into<[u8; 3]>>(mut self, i: usize, color: C) -> Self {
        self.colors[i] = color.into();
        self
    }

    /// Create a palette that is all one color. Not particularly useful on its own, but can be
    /// combined with `set()` to generate custom palettes.
    ///
    /// ```
    /// use dashing::resources::color::Palette;
    /// let p = Palette::mono::<16, _>([128, 128, 128]);
    ///
    /// for i in 0..16 {
    ///     assert_eq!(p[i], [128, 128, 128]);
    /// }
    /// ```
    pub fn mono<const N: usize, C: Into<[u8; 3]>>(color: C) -> Self {
        Palette {
            colors: vec![color.into(); N].into_boxed_slice(),
        }
    }

    /// Get the size of this Palette.
    pub fn size(&self) -> usize {
        self.colors.len()
    }

    pub(crate) fn as_texture_data(&self) -> Box<[u8]> {
        let mut result = vec![0; self.size() * 4];
        for (i, o) in self.colors.iter().zip(result.chunks_mut(4)) {
            o[0] = i[0];
            o[1] = i[1];
            o[2] = i[2];
            o[3] = 255;
        }
        result.into_boxed_slice()
    }
}

impl Default for Palette {
    /// Create a palette based on the CGA palette.
    fn default() -> Self {
        Palette::new([
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
        ])
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
