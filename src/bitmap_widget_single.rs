use std::fmt::Debug;

pub use crate::multimap::{
    BitMapText, ColorWithThickness, CoordinatePoint, Data, FontOptions, Overlay, RenderProblem,
};
use crate::MultiBitmapWidget;
use egui::Color32 as Color;

/// Hover type
#[derive(Debug, Clone)]
pub enum MapPosition {
    /// Mouse is not hovering over widget
    NotHovering,
    /// Mouse is hovering over widget, but outside of data area
    NoData(CoordinatePoint),
    /// Mouse is hoverinlg over data area, containing the point in data coordinates
    Pixel(CoordinatePoint),
    /// Mouse is over Colorbar
    Colorbar(f32),
}
impl From<crate::MultiMapPosition<()>> for MapPosition {
    fn from(value: crate::MultiMapPosition<()>) -> Self {
        match value {
            crate::MultiMapPosition::NotHovering => MapPosition::NotHovering,
            crate::MultiMapPosition::NoData((), pos) => MapPosition::NoData(pos),
            crate::MultiMapPosition::Pixel((), pos) => MapPosition::Pixel(pos),
            crate::MultiMapPosition::Colorbar(c) => MapPosition::Colorbar(c),
        }
    }
}

/// This is a bitmap widget, the main type of this crate
pub struct BitmapWidget {
    map: MultiBitmapWidget<()>,
}

impl BitmapWidget {
    /// Fetch array which is currently shown
    pub fn currently_showing(&self) -> crate::CoordinateRect {
        self.map.currently_showing()
    }
    /// Main Constructor. This assumes that the data coordinates are linearly and axis-aligned to the bitmap, but the left-top corner can be adjusted for each subplot
    pub fn with_settings(data: Data<Color>, settings: crate::MultiBitmapWidgetSettings) -> Self {
        Self {
            map: MultiBitmapWidget::with_settings(vec![((), data)], settings),
        }
    }

    /// Check if widget is hovered
    pub fn hover(&self) -> MapPosition {
        self.map.hover().into()
    }
    /// Check if there was a problem during the last rendering pass
    pub fn problem(&self) -> Option<RenderProblem> {
        self.map.problem()
    }

    /// Show widget
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.map.ui(ui)
    }
 
    /// Get the currently selected points
    pub fn selected(&self) -> impl ExactSizeIterator<Item = &CoordinatePoint> {
        self.map.selected()
    }
}
