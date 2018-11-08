use crate::graphics::SpriteLayer;
use crate::resources::color::Palette;
use crate::resources::sprite::SpriteMap;

/// Sprites necessary for these drawing routines.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sprite {
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
    S: SpriteMap<Sprite>,
{
    let mut out = SpriteLayer::new(width, height);
    out[(0, 0)].sprite = sprite_map.map(Sprite::TopLeftCorner);
    out[(width - 1, 0)].sprite = sprite_map.map(Sprite::TopRightCorner);
    out[(0, height - 1)].sprite = sprite_map.map(Sprite::BottomLeftCorner);
    out[(width - 1, height - 1)].sprite = sprite_map.map(Sprite::BottomRightCorner);

    for i in 1..width - 1 {
        out[(i, 0)].sprite = sprite_map.map(Sprite::HorizontalTop);
        out[(i, height - 1)].sprite = sprite_map.map(Sprite::HorizontalBottom);
    }
    for i in 1..height - 1 {
        out[(0, i)].sprite = sprite_map.map(Sprite::VerticalLeft);
        out[(width - 1, i)].sprite = sprite_map.map(Sprite::VerticalRight);
    }
    for x in 1..width - 1 {
        for y in 1..height - 1 {
            out[(x, y)].sprite = sprite_map.map(Sprite::SolidFill);
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
    S: SpriteMap<Sprite> + Copy,
{
    let mut base = rect(sprite_map, width, height, p);
    if height < 4 {
        return base;
    }
    base[(0, 2)].sprite = sprite_map.map(Sprite::TeeRight);
    base[(width - 1, 2)].sprite = sprite_map.map(Sprite::TeeLeft);
    for i in 1..width - 1 {
        base[(i, 2)].sprite = sprite_map.map(Sprite::HorizontalBottom);
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
