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

//! `dashing` is a library for building roguelike games.
//!
//! # Example
//!
//! ```no_run
//! use dashing::*;
//! use glutin;
//!
//! // Screen dimensions in characters.
//! const WIDTH: u32 = 21;
//! const HEIGHT: u32 = 3;
//!
//! // Client code must implement a "driver", which is a combination of input handler and
//! // interface to the renderer.
//! struct ExampleDriver {
//!     // SpriteLayer is a convenient abstraction for representing an orthogonal region of the
//!     // screen.
//!     root_layer: graphics::drawing::SpriteLayer,
//!     message: String,
//! }
//!
//! impl EngineDriver for ExampleDriver {
//!     // The driver handles all the user input events the window receives. The `handle_input`
//!     // method needs to be lightweight, because it blocks the render thread.
//!     fn handle_input(&mut self, e: glutin::Event) -> EngineSignal {
//!         match e {
//!             glutin::Event::WindowEvent { event: w, .. } => match w {
//!                 glutin::WindowEvent::CloseRequested | glutin::WindowEvent::Destroyed => {
//!                     return EngineSignal::Halt;
//!                 }
//!                 _ => {}
//!             },
//!             _ => {}
//!         }
//!         EngineSignal::Continue
//!     }
//!
//!     // After the engine processes all pending events, `process_frame` is called. This is where
//!     // you would update the screen, as well as run any per-frame processing your game
//!     // requires.
//!     fn process_frame<R>(&mut self, renderer: &mut R) -> EngineSignal
//!     where
//!         R: dashing::graphics::render::RenderInterface,
//!     {
//!         // Clear the sprite layer.
//!         self.root_layer.clear();
//!         // Print the message to the screen.
//!         for (i, c) in self.message.chars().enumerate() {
//!             self.root_layer[WIDTH as usize + 1 + i] = graphics::drawing::SpriteCell {
//!                 palette: resources::color::Palette::mono([0, 0, 0]).set(1, [255, 255, 255]),
//!                 sprite: c as u32,
//!                 transparent: false,
//!             };
//!         }
//!         // Update the renderer.
//!         renderer.update(self.root_layer.iter());
//!
//!         EngineSignal::Continue
//!     }
//! }
//!
//! pub fn main() {
//!     let pixels: Vec<u8> = vec![]; // Add your own texture here
//!     let tex =
//!         resources::sprite::SpriteTexture::new_from_pixels(&pixels[..], 128, 128, 8, 8, 256)
//!         .unwrap();
//!     let window_builder = window::WindowBuilder::new("dashing", WIDTH, HEIGHT, &tex);
//!     let driver = ExampleDriver {
//!         root_layer: graphics::drawing::SpriteLayer::new(WIDTH as usize, HEIGHT as usize),
//!         message: String::from("Swash your buckles!"),
//!     };
//!
//!     let engine = dashing::Engine::new(window_builder, driver).unwrap();
//!
//!     // `run_forever()` consumes the `Engine` object, so it cannot be used after this returns.
//!     engine.run_forever();
//! }
//!
//! ```
//!
//! # Roadmap
//! ## Features to be implemented
//!
//! * Input handling
//!   * Don't use glutin/winit event types in the public interface
//! * GUI library
//!   * Splash screen support
//! * Serialization/persistence framework
//! * Graphics improvements
//!   * User-specified shaders
//!   * Animated sprites
//!   * Hotswapping fonts
//!   * Dynamic zoom
//! * Resource management system
//!   * Build sprite map textures at runtime
//! * Audio
//! * Parallelism

#![recursion_limit = "72"]
#![deny(missing_docs)]
#![allow(dead_code)]

/// Routines for creating and managing the game window.
pub mod window;

/// API for interacting with the low-level rendering system.
pub mod graphics;

/// Functionality for loading and managing game data, such as sprite textures.
pub mod resources;

/// Keyboard & mouse input handling.
pub mod input;

/// Functionality for building in-game UIs.
pub mod ui;

/// Entity-component-system library
pub mod ecs;

/// Signals to indicate whether the engine should keep running or halt.
#[derive(PartialEq, Eq, Debug)]
pub enum EngineSignal {
    /// Continue running.
    Continue,
    /// Halt the main loop.
    Halt,
}

impl EngineSignal {
    fn update(&mut self, c: EngineSignal) {
        if c == EngineSignal::Halt {
            *self = EngineSignal::Halt
        }
    }
}

/// `Engine` is an abstraction of the main event loop for the game. Its functionality
/// encompasses:
/// - Creating the main window
/// - Polling for events and dispatching them to event handlers
/// - Calling the renderer
///
/// Because most of these actions need to be thread-local, this struct itself is neither `Send` nor
/// `Sync`, and `run_forever` *must* therefore be called on the main thread. However, this
/// restriction does not necessarily apply to user code as long as it does not touch the window or
/// renderer. TODO: provide abstractions for asynchronous inter-frame computation
pub struct Engine<E>
where
    E: EngineDriver,
{
    window: window::Window,
    driver: E,
}

impl<E> Engine<E>
where
    E: EngineDriver,
{
    /// Create a new `Engine`.
    pub fn new(
        window_builder: window::WindowBuilder,
        driver: E,
    ) -> Result<Self, window::WindowError> {
        Ok(Engine {
            window: window_builder.build()?,
            driver: driver,
        })
    }

    /// Run the main loop until one of the library hooks tells us to quit.
    pub fn run_forever(mut self) {
        let mut control = EngineSignal::Continue;
        while control == EngineSignal::Continue {
            control = (&mut self).run_once();
        }
    }

    /// Run the engine for a single frame.
    /// This allows clients to have more control over the main loop.
    pub fn run_once(&mut self) -> EngineSignal {
        let mut control = EngineSignal::Continue;
        let driver = &mut self.driver;
        self.window.event_loop_mut().poll_events(|e| {
            control.update(driver.handle_input(e));
        });
        if control == EngineSignal::Halt {
            return control;
        }
        if driver.process_frame(self.window.renderer_mut()) == EngineSignal::Halt {
            return EngineSignal::Halt;
        }

        self.window.render().unwrap();

        EngineSignal::Continue
    }
}

/// `EngineDriver` is the primary means by which an `Engine` communicates with client code. Clients
/// are expected to implement this trait themselves, but there are abstractions provided for
/// dealing with input in the [input] module.
pub trait EngineDriver {
    /// Handle an input event. This will be called for every event received by the main window, so
    /// it needs to be fast.
    fn handle_input(&mut self, e: glutin::Event) -> EngineSignal;
    /// Client hook for processing in the main loop. This gets called immediately before the
    /// renderer runs.
    fn process_frame<R>(&mut self, renderer: &mut R) -> EngineSignal
    where
        R: graphics::render::RenderInterface;
}
