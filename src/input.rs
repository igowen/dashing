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

pub use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::*,
};

use crate::geometry::Point;

/// Mouse event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseEvent {
    /// The cursor was moved to a new position.
    CursorMoved {
        /// Sprite-level position of the cursor.
        sprite_position: Point,
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
    /// A mouse button was pressed.
    Button {
        /// Pressed/released
        state: ElementState,
        /// Which button
        button: MouseButton,
    },
    // TODO: mouse wheel, etc.
}

/// Window-level events.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WindowEvent {
    /// The window was destroyed.
    Destroyed,
    /// The window has been requested to close.
    CloseRequested,
    /// The window gained or lost focus. The `bool` is set to true iff the window gained focus.
    Focused(bool),
}

/// Describes a generic event. Events are (generally) directly converted from an underlying window
/// manager or user input event provided by `winit`, but simplified. For example, most window
/// events have a `WindowId` to discern where the event originated; `dashing` creates and manages a
/// single window, so this is unnecessary (and discarded).
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Window-level event.
    Window(WindowEvent),

    /// Keyboard input.
    Keyboard(KeyEvent),

    /// Mouse input.
    Mouse(MouseEvent),
}

impl std::convert::TryFrom<winit::event::Event<()>> for Event {
    type Error = ();
    fn try_from(e: winit::event::Event<()>) -> Result<Self, Self::Error> {
        match &e {
            // Assumption: there is a single window, so we can safely discard fields used to
            // discern which window should receive the event.
            // Assumption: client code doesn't care what device the event originated from.
            winit::event::Event::WindowEvent { event: w, .. } => match w {
                // Easy passthrough cases.
                winit::event::WindowEvent::CloseRequested => {
                    Ok(Event::Window(WindowEvent::CloseRequested))
                }
                winit::event::WindowEvent::Destroyed => Ok(Event::Window(WindowEvent::Destroyed)),
                winit::event::WindowEvent::Focused(focused) => {
                    Ok(Event::Window(WindowEvent::Focused(*focused)))
                }
                winit::event::WindowEvent::CursorEntered { .. } => {
                    Ok(Event::Mouse(MouseEvent::CursorEntered))
                }
                winit::event::WindowEvent::CursorLeft { .. } => {
                    Ok(Event::Mouse(MouseEvent::CursorLeft))
                }
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    Ok(Event::Keyboard(event.clone()))
                }
                #[allow(deprecated)]
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    Ok(Event::Mouse(MouseEvent::Button {
                        state: *state,
                        button: *button,
                    }))
                }
                // We have to handle this one in Engine::run() directly, since it depends on a lot of
                // state that is not accessible here.
                winit::event::WindowEvent::CursorMoved { .. } => Err(()),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
