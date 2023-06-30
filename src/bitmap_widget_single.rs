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
    /// Main Constructor. This assumes that the data coordinates are linearly and axis-aligned to the bitmap, but the left-top corner can be adjusted for each subplot
    pub fn with_settings(data: Data<Color>, settings: crate::MultiBitmapWidgetSettings) -> Self {
        Self {
            map: MultiBitmapWidget::with_settings(vec![((), data)], settings),
        }
    }
    /// Get default state, in english
    pub fn default_state_english(&self) -> ShowStateSingle {
        ShowStateSingle {
            state: self.map.default_state_english(),
        }
    }
    /// Show widget
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut ShowStateSingle) {
        self.map.ui(ui, &mut state.state)
    }
}

/// This encodes the current state of the heatmap
pub struct ShowStateSingle {
    state: crate::bitmap_widget_multi::ShowState<()>,
}
impl ShowStateSingle {
    /// Select the given positions and only those
    pub fn make_selected(&mut self, selected:std::collections::HashSet<CoordinatePoint>){
        self.state.make_selected(selected)
    }
    /// Clear selected positions
    pub fn clear_selected(&mut self){
        self.state.clear_selected()
    }
    /// Get events
    pub fn events(&mut self) -> Vec<crate::Event<()>> {
        self.state.events()
    }
    /// Get the currently selected points
    pub fn selected(&self) -> &std::collections::HashSet<CoordinatePoint> {
        self.state.selected()
    }
    /// Fetch rectangle which is currently shown
    pub fn currently_showing(&self) -> Option<crate::CoordinateRect> {
        self.state.currently_showing()
    }
    /// Check if there was an issue will rendering
    pub fn render_problem(&self) -> Option<&RenderProblem> {
        self.state.render_problem()
    }
    /// Check if position was clicked
    pub fn clicked(&self) -> Option<MapPosition> {
        self.state.clicked().cloned().map(Into::into)
    }
    /// Check if position was clicked
    pub fn hover(&self) -> MapPosition {
        self.state.hover().clone().into()
    }
}
