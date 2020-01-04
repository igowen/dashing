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

use glutin;
use std::collections::HashMap;

pub use glutin::VirtualKeyCode;

use crate::Event;

/// KeyBinding is a specification for a keyboard shortcut.
#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq)]
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
            key: key,
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
    pub fn matches(&self, event: glutin::KeyboardInput) -> bool {
        if event.state == glutin::ElementState::Pressed {
            match event.virtual_keycode {
                None => false,
                Some(vk) => {
                    if vk == self.key {
                        self.shift == event.modifiers.shift
                            && self.ctrl == event.modifiers.ctrl
                            && self.alt == event.modifiers.alt
                            && self.meta == event.modifiers.logo
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    fn try_from_event(event: glutin::KeyboardInput) -> Option<KeyBinding> {
        if event.state == glutin::ElementState::Pressed {
            match event.virtual_keycode {
                None => None,
                Some(vk) => Some(KeyBinding {
                    key: vk,
                    shift: event.modifiers.shift,
                    ctrl: event.modifiers.ctrl,
                    alt: event.modifiers.alt,
                    meta: event.modifiers.logo,
                }),
            }
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
            command_queue: command_queue,
        }
    }

    /// Handle an event.
    pub fn dispatch(&self, event: &Event) -> Result<(), std::sync::mpsc::SendError<C>> {
        match event {
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

        assert!(kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState::default()
        }));

        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::X),
            modifiers: glutin::ModifiersState::default()
        }));

        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState {
                shift: true,
                ctrl: false,
                alt: false,
                logo: false
            }
        }));

        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState {
                shift: false,
                ctrl: true,
                alt: false,
                logo: false
            }
        }));

        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState {
                shift: false,
                ctrl: false,
                alt: true,
                logo: false
            }
        }));

        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState {
                shift: false,
                ctrl: false,
                alt: false,
                logo: true
            }
        }));
    }
}
