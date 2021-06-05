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

use log::info;

use crate::graphics::render;
use crate::resources::color::Color;
use crate::resources::sprite::SpriteTexture;

/// `WindowError` represents an error that occurred in the window system.
#[derive(Debug)]
pub enum WindowError {
    /// Generic error.
    GeneralError(String),
    /// Error from the renderer.
    RenderError(render::RenderError),
}

// TODO: Get rid of this, it really sucks.
impl<S> std::convert::From<S> for WindowError
where
    S: std::string::ToString,
{
    fn from(s: S) -> Self {
        WindowError::GeneralError(s.to_string())
    }
}

impl std::convert::From<render::RenderError> for WindowError {
    fn from(e: render::RenderError) -> Self {
        WindowError::RenderError(e)
    }
}

/// Enum for specifying the filter method used on the screen.
pub enum FilterMethod {
    /// Nearest neighbor filtering. Only looks good when the screen is an integral multiple of the
    /// unscaled screen size.
    NearestNeighbor,
    /// Bilinear filter.
    Linear,
}

impl From<FilterMethod> for wgpu::FilterMode {
    fn from(f: FilterMethod) -> Self {
        match f {
            FilterMethod::NearestNeighbor => wgpu::FilterMode::Nearest,
            FilterMethod::Linear => wgpu::FilterMode::Linear,
        }
    }
}

/// Helper for constructing windows.
pub struct WindowBuilder<'a> {
    window_title: &'a str,
    width: u32,
    height: u32,
    sprite_texture: &'a SpriteTexture,
    vsync: bool,
    resizable: bool,
    full_screen: bool,
    clear_color: Color,
    filter_method: FilterMethod,
}

impl<'a> WindowBuilder<'a> {
    /// Create a new `WindowBuilder` with the given width and height (measured in sprites, not
    /// pixels).
    ///
    /// Defaults:
    ///   - Vsync enabled
    ///   - Not resizable
    ///   - Not full screen
    ///   - Clear color 100% green
    ///   - Trilinear filtering
    pub fn new(
        window_title: &'a str,
        width: u32,
        height: u32,
        sprite_texture: &'a SpriteTexture,
    ) -> Self {
        WindowBuilder {
            window_title,
            width,
            height,
            sprite_texture,
            vsync: true,
            resizable: false,
            full_screen: false,
            clear_color: [0, 255, 0].into(),
            filter_method: FilterMethod::NearestNeighbor,
        }
    }

    /// Enable/disable vsync.
    pub fn with_vsync(mut self, enable: bool) -> Self {
        self.vsync = enable;

        self
    }

    /// Enable/disable window resizing.
    pub fn with_resizable(mut self, enable: bool) -> Self {
        self.resizable = enable;

        self
    }

    /// Enable full screen mode.
    pub fn enable_full_screen(mut self) -> Self {
        self.full_screen = true;

        self
    }

    /// Set the color that will be used to clear the screen. This color will be visible when the
    /// viewport's aspect ratio does not match the aspect ratio of the character display.
    pub fn with_clear_color(mut self, c: Color) -> Self {
        self.clear_color = c;

        self
    }

    /// Set the filter method used when scaling the screen.
    pub fn with_filter_method(mut self, f: FilterMethod) -> Self {
        self.filter_method = f;

        self
    }

    /// Build the window.
    pub fn build(self) -> Result<Window, WindowError> {
        // TODO: Don't create a window bigger than the display.
        let screen_width = (self.width * self.sprite_texture.sprite_width() as u32) as f32;
        let screen_height = (self.height * self.sprite_texture.sprite_height() as u32) as f32;
        info!("Screen dimensions {}x{}", screen_width, screen_height);
        // TODO: Figure out how to deal with hidpi
        let screen_dimensions = winit::dpi::LogicalSize::<f32>::from_physical(
            winit::dpi::PhysicalSize::new(screen_width, screen_height),
            1.0,
        );
        info!("logical size: {:?}", screen_dimensions);
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(self.window_title.to_string())
            .with_inner_size(screen_dimensions)
            .with_maximized(self.full_screen)
            .with_decorations(!self.full_screen)
            .with_resizable(self.resizable)
            .with_visible(false)
            .with_min_inner_size(winit::dpi::PhysicalSize::new(1, 1))
            .build(&event_loop)?;

        let renderer = crate::graphics::render::Renderer::new(
            Some(&window),
            (self.width as _, self.height as _),
            self.sprite_texture,
            self.clear_color,
            self.filter_method.into(),
            if self.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Mailbox
            },
        )?;

        Ok(Window {
            width: self.width,
            height: self.height,
            window,
            event_loop,
            renderer,
        })
    }
}

/// `Window` is responsible for creating and managing the game window and underlying GL context.
pub struct Window {
    // Handles to device resources we need to hold onto.
    pub(crate) renderer: render::Renderer,
    pub(crate) window: winit::window::Window,
    pub(crate) event_loop: winit::event_loop::EventLoop<()>,

    // Width & height of the window (in sprites).
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Window {
    /// Render one frame.
    pub(crate) fn render(&mut self) -> Result<(), WindowError> {
        unimplemented!();
    }

    /// Get a mutable reference to the underlying renderer.
    pub(crate) fn renderer_mut(&mut self) -> &mut render::Renderer {
        &mut self.renderer
    }

    /// Get an immutable reference to the underlying renderer.
    pub(crate) fn renderer(&self) -> &render::Renderer {
        &self.renderer
    }
}
