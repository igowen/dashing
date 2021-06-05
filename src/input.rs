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

use std::collections::HashMap;

pub use winit::event::{ElementState, MouseButton, VirtualKeyCode};

/// State of the modifier keys.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModifiersState {
    /// Control
    pub ctrl: bool,
    /// Alt
    pub alt: bool,
    /// Shift
    pub shift: bool,
    /// Meta/Logo
    pub meta: bool,
}

impl From<winit::event::ModifiersState> for ModifiersState {
    fn from(m: winit::event::ModifiersState) -> Self {
        ModifiersState {
            ctrl: m.ctrl(),
            alt: m.alt(),
            shift: m.shift(),
            meta: m.logo(),
        }
    }
}

/// Keyboard event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyboardEvent {
    /// Keyboard input interpreted as a character.
    Character(char),
    /// Raw key input.
    KeyPress {
        /// The raw scancode of the key.
        scancode: u32,
        /// Whether the key was pressed or released.
        state: ElementState,
        /// If the key can be interpreted as a KeyCode, it will be populated here.
        virtual_keycode: Option<VirtualKeyCode>,
        /// The modifiers active for this event.
        modifiers: ModifiersState,
    },
}

impl From<winit::event::KeyboardInput> for KeyboardEvent {
    fn from(k: winit::event::KeyboardInput) -> Self {
        #[allow(deprecated)]
        KeyboardEvent::KeyPress {
            scancode: k.scancode,
            state: k.state,
            virtual_keycode: k.virtual_keycode,
            modifiers: k.modifiers.into(),
        }
    }
}

/// Mouse event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseEvent {
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
    /// A mouse button was pressed.
    Button {
        /// Pressed/released
        state: ElementState,
        /// Which button
        button: MouseButton,
        /// Modifier key state
        modifiers: ModifiersState,
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
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    /// Window-level event.
    Window(WindowEvent),

    /// Keyboard input.
    Keyboard(KeyboardEvent),

    /// Mouse input.
    Mouse(MouseEvent),
}

impl std::convert::TryFrom<winit::event::Event<'_, ()>> for Event {
    type Error = ();
    fn try_from(e: winit::event::Event<'_, ()>) -> Result<Self, Self::Error> {
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
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    Ok(Event::Keyboard((*input).into()))
                }
                #[allow(deprecated)]
                winit::event::WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => Ok(Event::Mouse(MouseEvent::Button {
                    state: *state,
                    button: *button,
                    modifiers: (*modifiers).into(),
                })),
                // We have to handle this one in Engine::run() directly, since it depends on a lot of
                // state that is not accessible here.
                winit::event::WindowEvent::CursorMoved { .. } => Err(()),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
/// KeyBinding is a specification for a keyboard shortcut.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBinding {
    key: VirtualKeyCode,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

impl KeyBinding {
    /// Create a new KeyBinding. All modifiers are set to false initially; use
    /// `ctrl()`/`shift()`/`alt()`/`meta()` to build bindings that use modifiers.
    pub fn new(key: VirtualKeyCode) -> Self {
        KeyBinding {
            key,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Set `ctrl` to true.
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// Set `shift` to true.
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    /// Set `alt` to true.
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }
    /// Set `meta` to true.
    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }

    /// Returns `true` iff the provided raw keyboard event matches this binding.
    pub fn matches(&self, event: winit::event::KeyboardInput) -> bool {
        if event.state == winit::event::ElementState::Pressed {
            #[allow(deprecated)]
            match event.virtual_keycode {
                None => false,
                Some(vk) => {
                    if vk == self.key {
                        self.shift == event.modifiers.shift()
                            && self.ctrl == event.modifiers.ctrl()
                            && self.alt == event.modifiers.alt()
                            && self.meta == event.modifiers.logo()
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    // TODO: impl TryFrom instead.
    fn try_from_event(event: winit::event::KeyboardInput) -> Option<KeyBinding> {
        if event.state == winit::event::ElementState::Pressed {
            #[allow(deprecated)]
            event.virtual_keycode.map(|vk| KeyBinding {
                key: vk,
                shift: event.modifiers.shift(),
                ctrl: event.modifiers.ctrl(),
                alt: event.modifiers.alt(),
                meta: event.modifiers.logo(),
            })
        } else {
            None
        }
    }
}

/// Build input bindings based on message queues for multithreaded event handling.
/// The type parameter `C` represents a "command" that is sent over the channel.
/// TODO: mouse events
pub struct EventDispatcher<C> {
    key_bindings: HashMap<KeyBinding, C>,
    window_close_command: Option<C>,
    command_queue: std::sync::mpsc::SyncSender<C>,
}

impl<C: Clone> EventDispatcher<C> {
    /// Set a key binding.
    pub fn bind(&mut self, key: KeyBinding, command: C) {
        self.key_bindings.insert(key, command);
    }

    /// Set command to send when the window is closed.
    pub fn set_window_close_command(&mut self, command: C) {
        self.window_close_command = Some(command);
    }

    /// Create a new dispatcher.
    pub fn new(command_queue: std::sync::mpsc::SyncSender<C>) -> Self {
        EventDispatcher {
            key_bindings: HashMap::<KeyBinding, C>::new(),
            window_close_command: None,
            command_queue,
        }
    }

    /// Handle an event.
    pub fn dispatch(&self, event: &Event) -> Result<(), std::sync::mpsc::SendError<C>> {
        match event {
            /*
            Event::KeyboardInput(ref ke) => {
                if let Some(binding) = KeyBinding::try_from_event(*ke) {
                    if let Some(command) = self.key_bindings.get(&binding) {
                        self.command_queue.send(command.clone())?;
                    }
                }
            }
            Event::WindowDestroyed | Event::WindowCloseRequested => {
                if let Some(ref command) = self.window_close_command {
                    self.command_queue.send(command.clone())?;
                }
            }
            */
            _ => {}
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn match_event() {
        use super::*;
        let kb = KeyBinding::new(VirtualKeyCode::Z);

        assert!(kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Z),
                modifiers: winit::event::ModifiersState::default()
            }
        ));

        assert!(!kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::X),
                modifiers: winit::event::ModifiersState::default()
            }
        ));

        assert!(!kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Z),
                modifiers: winit::event::ModifiersState::SHIFT
            }
        ));

        assert!(!kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Z),
                modifiers: winit::event::ModifiersState::CTRL
            }
        ));

        assert!(!kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Z),
                modifiers: winit::event::ModifiersState::ALT
            }
        ));

        assert!(!kb.matches(
            #[allow(deprecated)]
            winit::event::KeyboardInput {
                scancode: 0,
                state: winit::event::ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Z),
                modifiers: winit::event::ModifiersState::LOGO
            }
        ));
    }
}
