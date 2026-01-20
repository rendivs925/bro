pub mod types;
pub mod utils;
pub mod analysis;
pub mod adapters;
pub mod agent;
pub mod cli;
pub mod confirmation;
pub mod editor;
pub mod session;

// TODO: Web module temporarily disabled due to dependency issues
// pub mod web;

#[cfg(feature = "tui")]
pub mod tui;
