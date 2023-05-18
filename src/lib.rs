//! Navigatable bitmap widget for egui
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    variant_size_differences
)]

mod bitmap_data;
mod bitmap_widget_multi;
mod bitmap_widget_single;
/// This contains some default colors and further color-related types, like gradients
pub mod colors;
mod font;
mod multimap;
pub use bitmap_data::HeatmapData;

/// Some font-related types
pub use font::{BitMapText, Font, FontOptions};
/// Color type: egui::Color32
pub type Color = egui::Color32;
pub use bitmap_widget_multi::{
    ColorWithThickness, CoordinatePoint, CoordinateRect, Data, Event, MultiBitmapWidget,
    MultiBitmapWidgetSettings, MultiMapPosition, Overlay,
};

pub use bitmap_widget_single::{BitmapWidget, MapPosition};
