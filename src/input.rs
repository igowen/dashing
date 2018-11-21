use glutin;
use std::collections::HashMap;

pub use glutin::VirtualKeyCode;

/// KeyBinding is a specification for a keyboard shortcut.
#[derive(Hash, Clone, Debug, PartialEq, Eq)]
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

impl<C> EventDispatcher<C>
where
    C: Copy,
{
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
    pub fn dispatch(&self, event: glutin::Event) -> Result<(), std::sync::mpsc::SendError<C>> {
        match event {
            glutin::Event::WindowEvent { event: w, .. } => match w {
                glutin::WindowEvent::KeyboardInput { input: ke, .. } => {
                    if let Some(binding) = KeyBinding::try_from_event(ke) {
                        if let Some(command) = self.key_bindings.get(&binding) {
                            self.command_queue.send(*command)?;
                        }
                    }
                }
                glutin::WindowEvent::CloseRequested | glutin::WindowEvent::Destroyed => {
                    if let Some(command) = self.window_close_command {
                        self.command_queue.send(command)?;
                    }
                }
                _ => {}
            },
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
