//! Dashing is a library for building roguelike games.

#![deny(warnings)]
#![deny(missing_docs)]
#![allow(dead_code)]

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate image;
#[allow(unused)]
#[macro_use]
extern crate log;
extern crate sdl2;
extern crate time;

/// The `engine` module contains the interface for doing graphics, input, and sound.
pub mod engine;

//// The `resources` module contains methods for managing game data files, like textures and sounds.
//pub mod resources;

// Libraries used in tests.
#[cfg(test)]
extern crate offscreen_gl_context;
#[cfg(test)]
extern crate gleam;
#[cfg(test)]
extern crate euclid;
#[cfg(test)]
extern crate pretty_logger;
#[cfg(test)]
extern crate spectral;
