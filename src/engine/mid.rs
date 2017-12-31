use std;
use sdl2;

use engine::ll;

/// CharCell shouldn't be public, really.
#[derive(Copy, Clone, Debug)]
pub struct CharCell {
    /// Character in the cell.
    pub character: u32,
    /// Foreground color.
    pub fg_color: [f32; 4],
    /// Background color.
    pub bg_color: [f32; 4],
    /// Is this cell visible?
    pub transparent: bool,
}

impl CharCell {
    /// Get this cell's character code.
    pub fn get_character(&self) -> u32 {
        self.character
    }
    /// Get this cell's foreground color.
    pub fn get_fg_color(&self) -> [f32; 4] {
        self.fg_color
    }
    /// Get this cell's background color.
    pub fn get_bg_color(&self) -> [f32; 4] {
        self.bg_color
    }
    /// Returns whether this cell is transparent (i.e., visible) or not.
    pub fn is_transparent(&self) -> bool {
        self.transparent
    }
}

impl Default for CharCell {
    fn default() -> Self {
        CharCell {
            character: 0,
            fg_color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.0, 0.0, 0.0, 1.0],
            transparent: true,
        }
    }
}

#[derive(Clone)]
struct MidEngineLayer {
    cells: Box<[CharCell]>,
    width: usize,
    height: usize,
}

impl MidEngineLayer {
    fn new(width: u32, height: u32) -> Self {
        MidEngineLayer {
            cells: vec![CharCell::default(); (width * height) as usize].into_boxed_slice(),
            width: width as usize,
            height: height as usize,
        }
    }

    fn composite(&self, out: &mut MidEngineLayer) {
        for (p, q) in self.cells.iter().zip(out.cells.iter_mut()) {
            if !p.transparent {
                *q = *p;
            }
        }
    }

    fn reset(&mut self) {
        for c in self.cells.iter_mut() {
            *c = CharCell::default();
        }
    }

    fn set(&mut self, x: usize, y: usize, character: u32) {
        self.cells[y * self.width + x].character = character;
        self.cells[y * self.width + x].transparent = false;
    }

    fn set_color(&mut self, x: usize, y: usize, color: [f32; 3]) {
        self.cells[y * self.width + x].fg_color = [color[0], color[1], color[2], 1.0];
    }

    fn set_background_color(&mut self, x: usize, y: usize, color: [f32; 3]) {
        self.cells[y * self.width + x].bg_color = [color[0], color[1], color[2], 1.0];
    }

    fn iter(&self) -> std::slice::Iter<CharCell> {
        self.cells.iter()
    }
}

/// `MidEngine` is the 'mid-level' engine. It doesn't interact with OpenGL et al. at all, but
/// provides an additional layer of abstraction on top of the low-level engine.
pub struct MidEngine {
    ll_engine: ll::LLEngine,
    layers: Box<[MidEngineLayer]>,
    base_layer: MidEngineLayer,
}

impl MidEngine {
    /// Create a new `MidEngine` with the given width and height (in characters) and number of
    /// layers.
    pub fn new(
        window_title: &str,
        width: u32,
        height: u32,
        layers: u32,
    ) -> Result<Self, ll::LLEngineError> {
        Ok(MidEngine {
            ll_engine: ll::LLEngine::new(window_title, width, height)?,
            layers: vec![MidEngineLayer::new(width, height); layers as usize].into_boxed_slice(),
            base_layer: MidEngineLayer::new(width, height),
        })
    }

    /// Set the character at (x,y,z).
    pub fn set(&mut self, x: usize, y: usize, z: usize, character: u32) {
        self.layers[z].set(x, y, character);
    }

    /// Set the foreground color at (x,y,z).
    pub fn set_color(&mut self, x: usize, y: usize, z: usize, color: [f32; 3]) {
        self.layers[z].set_color(x, y, color);
    }

    /// Set the background color at (x,y,z).
    pub fn set_background_color(&mut self, x: usize, y: usize, z: usize, color: [f32; 3]) {
        self.layers[z].set_background_color(x, y, color);
    }

    /// Render one frame, and return an iterator over the events that have elapsed since the last
    /// frame.
    /// TODO: Don't use sdl2 types here.
    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, ll::LLEngineError> {
        self.base_layer.reset();
        for layer in self.layers.iter().rev() {
            layer.composite(&mut self.base_layer);
        }
        self.ll_engine.update(self.base_layer.iter());
        self.ll_engine.render()
    }
    /// Get the current frames per second. This is based on a rolling average, not the
    /// instantaneous measurement.
    pub fn get_fps(&self) -> f32 {
        self.ll_engine.get_fps()
    }
    /// Get the number of frames that have been rendered.
    pub fn get_frame_counter(&self) -> u32 {
        self.ll_engine.get_frame_counter()
    }
}
