//! Deterministic terminal rendering for generic fixtures and pack-driven product frames.

mod ansi;
mod halfblock;
mod inspectors;
mod labels;
mod layout;
mod legacy;
mod panels;
mod permission;
mod product;
mod text;
mod theme;
mod workshop;

pub use ansi::buffer_to_ansi;
pub use halfblock::ColorCapability;
pub use layout::{select_layout, LayoutMode};
pub use legacy::{
    buffer_to_string, render_frame, render_frame_string, DEFAULT_FRAME_HEIGHT, DEFAULT_FRAME_WIDTH,
};
pub use product::{render_product_frame, ProductWidget};

#[cfg(test)]
mod tests;
