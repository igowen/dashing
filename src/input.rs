use glutin;

pub use glutin::VirtualKeyCode;

/// KeyBinding is a specification for a keyboard shortcut.
#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyBinding {
    key: VirtualKeyCode,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

impl KeyBinding {
    fn new(key: VirtualKeyCode, shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        KeyBinding {
            key: key,
            shift: shift,
            ctrl: ctrl,
            alt: alt,
            meta: meta,
        }
    }

    fn matches(&self, event: glutin::KeyboardInput) -> bool {
        if event.state == glutin::ElementState::Released {
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
}

/// Event Handler
pub struct EventHandler {}
impl EventHandler {
    pub(crate) fn handle(&mut self, _e: glutin::Event) -> crate::EngineSignal {
        return crate::EngineSignal::Continue;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn match_event() {
        use super::*;
        let kb = KeyBinding::new(VirtualKeyCode::Z, false, false, false, false);
        assert!(kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Released,
            virtual_keycode: Some(VirtualKeyCode::Z),
            modifiers: glutin::ModifiersState::default()
        }));
        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Released,
            virtual_keycode: Some(VirtualKeyCode::X),
            modifiers: glutin::ModifiersState::default()
        }));
        assert!(!kb.matches(glutin::KeyboardInput {
            scancode: 0,
            state: glutin::ElementState::Released,
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
            state: glutin::ElementState::Released,
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
            state: glutin::ElementState::Released,
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
            state: glutin::ElementState::Released,
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
