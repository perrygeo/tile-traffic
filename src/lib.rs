//! `tile_traffic`
//! is a Rust library for measuring web map service performance
//!
//! At its core, tile_traffic generates realistic map API traffic and provides
//! summary statistics.
//!
//! for examples of how it's implemented,
//! see CLI and the Python API
//!
pub mod coordinates;
pub mod request_handler;
pub mod statistics;
pub mod strategies;
pub mod tui;
