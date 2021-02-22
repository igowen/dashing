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

/*
/// Keyboard & mouse input handling.
pub mod input;
*/

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

pub use winit::event::{ElementState, KeyboardInput, ModifiersState, MouseButton};

/// Describes a generic event. Events are (generally) directly converted from an underlying window
/// manager or user input event provided by `winit`, but simplified. For example, most window
/// events have a `WindowId` to discern where the event originated; `dashing` creates and manages a
/// single window, so this is unnecessary (and discarded).
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The window was destroyed.
    WindowDestroyed,
    /// The window has been requested to close.
    WindowCloseRequested,
    /// The window gained or lost focus. The `bool` is set to true iff the window gained focus.
    Focused(bool),
    /// Keyboard input. TODO: don't expose winit types here.
    KeyboardInput(KeyboardInput),
    /// The cursor was moved to a new position.
    CursorMoved {
        /// Sprite-level position of the cursor.
        sprite_position: (u32, u32),
        /* TODO: calculate this.
        /// Logical pixel location of the cursor (independent of the actual window size).
        pixel_position: (u32, u32),
        */
        /// The unprocessed location straight from the underlying event.
        absolute_position: (f64, f64),
    },

    /// The mouse cursor entered the window.
    CursorEntered,
    /// The mouse cursor left the window.
    CursorLeft,
    /// A mouse button was pressed. TODO: don't expose winit types here.
    MouseButton {
        /// Pressed/released
        state: ElementState,
        /// Which button
        button: MouseButton,
        /// Modifier key state
        modifiers: ModifiersState,
    },
    // TODO: mouse wheel, etc.
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

        let (mut renderer, winit_window, event_loop) =
            (window.renderer, window.window, window.event_loop);
        let winit_window_id = winit_window.id();
        use winit::event::{Event, WindowEvent};
        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                renderer.render_frame().unwrap();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == winit_window_id => {
                debug!("{:?}", event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                if driver.process_frame(&mut renderer) == EngineSignal::Halt {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }

                winit_window.request_redraw();
            }
            _ => {}
        })
    }

    /*
    /// Run the engine for a single frame.
    /// This allows clients to have more control over the main loop.
    pub fn run_once(&mut self) -> EngineSignal {
        let mut control = EngineSignal::Continue;
        let driver = &mut self.driver;
        let mut new_size: Option<wgpu::dpi::LogicalSize> = Option::None;
        let mut cursor_position: Option<wgpu::dpi::LogicalPosition> = Option::None;
        self.window.event_loop_mut().poll_events(|e| {
            let interpreted_event = match &e {
                // Assumption: there is a single window, so we can safely discard fields used to
                // discern which window should receive the event.
                // Assumption: client code doesn't care what device the event originated from.
                winit::event::Event::WindowEvent { event: w, .. } => match w {
                    // Easy passthrough cases.
                    winit::event::WindowEvent::CloseRequested => Some(Event::WindowCloseRequested),
                    winit::event::WindowEvent::Destroyed => Some(Event::WindowDestroyed),
                    winit::event::WindowEvent::CursorEntered { .. } => Some(Event::CursorEntered),
                    winit::event::WindowEvent::CursorLeft { .. } => Some(Event::CursorLeft),
                    winit::event::WindowEvent::Focused(focused) => Some(Event::Focused(*focused)),
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        //Some(Event::KeyboardInput(*input))
                        None
                    }
                    winit::event::WindowEvent::MouseInput {
                        state,
                        button,
                        modifiers,
                        ..
                    } => {
                        //Some(Event::MouseButton {
                        //    state: *state,
                        //    button: *button,
                        //    modifiers: *modifiers,
                        //})
                        None
                    },
                    // Resize is handled below.
                    winit::event::WindowEvent::Resized(s) => {
                        new_size = Some(*s);
                        None
                    }
                    // The sprite position calculation happens after all events have been
                    // processed.
                    winit::event::WindowEvent::CursorMoved { position: p, .. } => {
                        cursor_position = Some(*p);
                        None
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some(ie) = interpreted_event {
                control.update(driver.handle_input(ie));
            }
        });

        let hidpi_factor = self
            .window
            .window
            .window()
            .get_current_monitor()
            .get_hidpi_factor();
        if let Some(s) = new_size {
            /*
            gfx_window_glutin::update_views(
                &self.window.window,
                &mut self.window.renderer.screen_pipeline_data.out,
                &mut self.window.renderer.depth_view,
            );
            self.window.window.resize(s.to_physical(hidpi_factor));
            */
        }
        if let Some(p) = cursor_position {
            let (ax, ay) = self.window.renderer.aspect_ratio;
            let glutin::dpi::PhysicalPosition { x, y } = p.to_physical(hidpi_factor);
            if let Some(size) = self.window.window.window().get_inner_size() {
                let glutin::dpi::PhysicalSize {
                    width: screen_w,
                    height: screen_h,
                } = size.to_physical(hidpi_factor);

                let target_w = std::cmp::min(screen_w as usize, (screen_h as usize * ax) / ay);
                let target_h = std::cmp::min(screen_h as usize, (screen_w as usize * ay) / ax);
                let offs_x;
                let offs_y;
                if target_w < screen_w as usize {
                    offs_x = (screen_w - target_w as f64) / 2.0;
                    offs_y = 0.0;
                } else {
                    // target_h < screen_h
                    offs_x = 0.0;
                    offs_y = (screen_h - target_h as f64) / 2.0;
                }

                if x > offs_x
                    && y > offs_y
                    && x < offs_x + target_w as f64
                    && y < offs_y + target_h as f64
                {
                    let xp = (x - offs_x) as f32;
                    let yp = (y - offs_y) as f32;

                    let sx = target_w as f32 / self.window.width as f32;
                    let sy = target_h as f32 / self.window.height as f32;

                    let xs = (xp / sx) as u32;
                    let ys = (yp / sy) as u32;

                    /*
                    control.update(driver.handle_input(Event::CursorMoved {
                        sprite_position: (xs, ys),
                        absolute_position: (x, y),
                    }));
                    */
                }
            }
        }

        if control == EngineSignal::Halt {
            return control;
        }
        if driver.process_frame(self.window.renderer_mut()) == EngineSignal::Halt {
            return EngineSignal::Halt;
        }

        self.window.render().unwrap();

        EngineSignal::Continue
    }
            */
}

/// `Driver` is the primary means by which an `Engine` communicates with client code. Clients
/// are required to implement this trait themselves, but there are abstractions provided for
/// dealing with input in the [input] module.
pub trait Driver {
    /// Handle an input event. This will be called for every event received by the main window, so
    /// it needs to be fast.
    fn handle_input(&mut self, event: Event) -> EngineSignal;

    /// Client hook for processing in the main loop. This gets called immediately before the
    /// renderer runs, but after all pending events have been processed via `handle_input()`.
    fn process_frame<R>(&mut self, renderer: &mut R) -> EngineSignal
    where
        R: graphics::render::RenderInterface;
}
