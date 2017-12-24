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
extern crate sdl2;
extern crate time;

/// The `engine` module contains the interface for doing graphics, input, and sound.
pub mod engine;
