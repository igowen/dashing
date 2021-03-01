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
//!
//! # Roadmap
//! ## Features to be implemented
//!
//! * Input handling
//!   * Don't use winit event types in the public interface
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

#[cfg(feature = "ecs")]
#[macro_use]
pub mod ecs;

use log::debug;

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
pub struct Engine<D>
where
    D: Driver,
{
    window: window::Window,
    driver: D,
}

impl<D> Engine<D>
where
    D: Driver + 'static,
{
    /// Create a new `Engine`.
    pub fn new(
        window_builder: window::WindowBuilder,
        driver: D,
    ) -> Result<Self, window::WindowError> {
        Ok(Engine {
            window: window_builder.build()?,
            driver: driver,
        })
    }

    /// Run the main loop until one of the library hooks tells us to quit.
    pub fn run(self) -> ! {
        let (window, mut driver) = (self.window, self.driver);
        window.window.set_visible(true);

        let (width, height, mut renderer, winit_window, event_loop) = (
            window.width,
            window.height,
            window.renderer,
            window.window,
            window.event_loop,
        );
        let winit_window_id = winit_window.id();
        event_loop.run(move |event, _, control_flow| {
            match event {
                winit::event::Event::RedrawRequested(_) => {
                    renderer.render_frame().unwrap();
                }
                winit::event::Event::WindowEvent {
                    ref event,
                    window_id,
                } => {
                    if window_id == winit_window_id {
                        //debug!("{:?}", event);
                        match event {
                            winit::event::WindowEvent::Resized(physical_size) => {
                                renderer.resize(*physical_size);
                            }
                            winit::event::WindowEvent::ScaleFactorChanged {
                                new_inner_size,
                                ..
                            } => {
                                renderer.resize(**new_inner_size);
                            }
                            winit::event::WindowEvent::CursorMoved { position, .. } => {
                                let (ax, ay) = renderer.aspect_ratio;
                                let winit::dpi::PhysicalPosition { x: xf, y: yf } = position;
                                let (x, y) = (*xf as u32, *yf as u32);
                                let winit::dpi::PhysicalSize {
                                    width: screen_w,
                                    height: screen_h,
                                } = winit_window.inner_size(); // = size.to_physical(scale_factor);

                                let target_w = std::cmp::min(screen_w, (screen_h * ax) / ay);
                                let target_h = std::cmp::min(screen_h, (screen_w * ay) / ax);
                                let offs_x;
                                let offs_y;
                                if target_w < screen_w {
                                    offs_x = (screen_w - target_w) / 2;
                                    offs_y = 0;
                                } else {
                                    // target_h < screen_h
                                    offs_x = 0;
                                    offs_y = (screen_h - target_h) / 2;
                                }

                                if x > offs_x
                                    && y > offs_y
                                    && x < offs_x + target_w
                                    && y < offs_y + target_h
                                {
                                    let xp = (x - offs_x) as f32;
                                    let yp = (y - offs_y) as f32;

                                    let sx = target_w as f32 / width as f32;
                                    let sy = target_h as f32 / height as f32;

                                    let xs = (xp / sx) as u32;
                                    let ys = (yp / sy) as u32;

                                    let e = input::Event::Mouse(input::MouseEvent::CursorMoved {
                                        sprite_position: (xs, ys),
                                        absolute_position: (*xf, *yf),
                                    });
                                    debug!("{:?}", e);
                                    if driver.handle_input(e) == EngineSignal::Halt {
                                        *control_flow = winit::event_loop::ControlFlow::Exit;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                winit::event::Event::MainEventsCleared => {
                    if driver.process_frame(&mut renderer) == EngineSignal::Halt {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }

                    winit_window.request_redraw();
                }
                _ => {}
            }
            if let Ok(e) = std::convert::TryInto::<input::Event>::try_into(event) {
                debug!("{:?}", e);
                if driver.handle_input(e) == EngineSignal::Halt {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            }
        })
    }
}

/// `Driver` is the primary means by which an `Engine` communicates with client code. Clients
/// are required to implement this trait themselves, but there are abstractions provided for
/// dealing with input in the [input] module.
pub trait Driver {
    /// Handle an input event. This will be called for every event received by the main window, so
    /// it needs to be fast.
    #[must_use]
    fn handle_input(&mut self, event: input::Event) -> EngineSignal;

    /// Client hook for processing in the main loop. This gets called immediately before the
    /// renderer runs, but after all pending events have been processed via `handle_input()`.
    #[must_use]
    fn process_frame<R>(&mut self, renderer: &mut R) -> EngineSignal
    where
        R: graphics::render::RenderInterface;
}
