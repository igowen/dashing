use std;
use sdl2;

use engine::ll;

#[derive(Copy, Clone, Debug)]
pub struct CharCell {
    character: u32,
    fg_color: [f32; 4],
    bg_color: [f32; 4],
    transparent: bool,
}

impl CharCell {
    pub fn get_character(&self) -> u32 {
        self.character
    }
    pub fn get_fg_color(&self) -> [f32; 4] {
        self.fg_color
    }
    pub fn get_bg_color(&self) -> [f32; 4] {
        self.bg_color
    }
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

pub struct MidEngine {
    ll_engine: ll::LLEngine,
    layers: Box<[MidEngineLayer]>,
    base_layer: MidEngineLayer,
}

impl MidEngine {
    pub fn new(width: u32, height: u32, layers: u32) -> Result<Self, ll::LLEngineError> {
        Ok(MidEngine {
            ll_engine: ll::LLEngine::new(width, height)?,
            layers: vec![MidEngineLayer::new(width, height); layers as usize].into_boxed_slice(),
            base_layer: MidEngineLayer::new(width, height),
        })
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, character: u32) {
        self.layers[z].set(x, y, character);
    }

    pub fn set_color(&mut self, x: usize, y: usize, z: usize, color: [f32; 3]) {
        self.layers[z].set_color(x, y, color);
    }

    pub fn set_background_color(&mut self, x: usize, y: usize, z: usize, color: [f32; 3]) {
        self.layers[z].set_background_color(x, y, color);
    }

    pub fn render(&mut self) -> Result<sdl2::event::EventPollIterator, ll::LLEngineError> {
        self.base_layer.reset();
        for layer in self.layers.iter().rev() {
            layer.composite(&mut self.base_layer);
        }
        self.ll_engine.update(self.base_layer.iter());
        self.ll_engine.render()
    }
    pub fn get_fps(&self) -> f64 {
        self.ll_engine.get_fps()
    }
    pub fn get_frame_counter(&self) -> u32 {
        self.ll_engine.get_frame_counter()
    }
}
