mod gamma_multiplyable;
pub use gamma_multiplyable::{BitMapDrawable, GammyMultiplyable};

pub use crate::font::{BitMapText, Font, FontOptions};
pub enum KeyBoardDirection {
    Up,
    Down,
    Left,
    Right,
}
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct MultimapState<Key: Eq + std::hash::Hash> {
    pub to_plot: std::collections::HashMap<Key, bool>,
    pub selected: std::collections::HashSet<CoordinatePoint>,
    pub shown_rectangle: Option<ShowRect>,
}

impl<Key: std::hash::Hash + Eq> MultimapState<Key> {
    fn to_plot(&self, key: &Key) -> bool {
        self.to_plot.get(key).cloned().unwrap_or(true)
    }
    pub(crate) fn currently_showing(&self) -> Option<CoordinateRect> {
        if let Some(ShowRect {
            left_top,
            right_bottom,
        }) = &self.shown_rectangle
        {
            Some(CoordinateRect {
                left_top: left_top - &CoordinatePoint { x: 0, y: 0 },
                right_bottom: right_bottom - &CoordinatePoint { x: 0, y: 0 },
            })
        } else {
            None
        }
    }
}
/// This is a point, using the user-given coordinate system
#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct CoordinatePoint {
    /// Column
    pub x: i32,
    /// Row
    pub y: i32,
}

/// This is a offset between two points, in user-given coordinates
#[derive(Debug)]
pub struct CoordinateVec {
    /// Column
    pub x: usize,
    /// Row
    pub y: usize,
}

pub struct MultiMapPoint {
    pub x: usize,
    pub y: usize,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct ShowPoint {
    x: i32,
    y: i32,
}
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ShowRect {
    left_top: ShowPoint,
    // this is right below of the last point, similiar to that an array length points "behind" the array
    right_bottom: ShowPoint,
}

/// This is a rectangle in the user-given coordinate system.
#[derive(Debug, PartialEq)]
pub struct CoordinateRect {
    /// Left top starting point of rectangle
    pub left_top: CoordinatePoint,
    /// This is right below of the last point, similiar to that an array length points "behind" the array
    pub right_bottom: CoordinatePoint,
}
impl CoordinateRect {
    fn delta(&self) -> CoordinateVec {
        &self.right_bottom - &self.left_top
    }
}
impl std::ops::Add<CoordinateVec> for &CoordinatePoint {
    type Output = CoordinatePoint;

    fn add(self, rhs: CoordinateVec) -> Self::Output {
        CoordinatePoint {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
        }
    }
}
impl std::ops::Sub<&CoordinatePoint> for &CoordinatePoint {
    type Output = CoordinateVec;

    fn sub(self, rhs: &CoordinatePoint) -> Self::Output {
        CoordinateVec {
            x: (self.x - rhs.x) as usize,
            y: (self.y - rhs.y) as usize,
        }
    }
}
impl std::ops::Sub<&CoordinatePoint> for &ShowRect {
    type Output = CoordinateRect;

    fn sub(self, rhs: &CoordinatePoint) -> Self::Output {
        CoordinateRect {
            left_top: &self.left_top - rhs,
            right_bottom: &self.right_bottom - rhs,
        }
    }
}
impl std::ops::Sub<&CoordinatePoint> for &ShowPoint {
    type Output = CoordinatePoint;

    fn sub(self, rhs: &CoordinatePoint) -> Self::Output {
        CoordinatePoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
struct RenderPoint {
    coordinate: CoordinatePoint,
    is_boundary: bool,
}

/// Overlay text, which is shown once user zooms in enough
pub struct Overlay {
    font: FontOptions,
    overlay_indices: std::collections::HashMap<CoordinatePoint, usize>,
    overlay_bitmaps: Vec<BitMapText>,
    show_coordinates: bool,
    title: String,
}
impl Overlay {
    /// Constructor
    pub fn new(
        font: FontOptions,
        show_coordinates: bool,
        overlay_text: std::collections::HashMap<CoordinatePoint, String>,
        title: &str,
    ) -> Option<Self> {
        //let title = font.render(title)?;
        let mut overlay_indices = std::collections::HashMap::default();
        let mut overlay_bitmaps = Vec::default();
        let mut overlay_strings = Vec::default();
        for (k, s) in overlay_text {
            let index = if let Some(index) = overlay_strings.iter().position(|x| x == &s) {
                index
            } else {
                let bitmap = font.render(&s)?;
                if let Some(index) = overlay_bitmaps.iter().position(|x| x == &bitmap) {
                    index
                } else {
                    let index = overlay_bitmaps.len();
                    overlay_bitmaps.push(bitmap);
                    overlay_strings.push(s);
                    index
                }
            };
            overlay_indices.insert(k, index);
        }
        Some(Self {
            font,
            overlay_indices,
            overlay_bitmaps,
            show_coordinates,
            title: title.to_string(),
        })
    }
    /// Create an exampleary overlay
    pub fn example(first_coordinate: &CoordinatePoint) -> Self {
        let mut overlay = std::collections::HashMap::<CoordinatePoint, _>::default();
        overlay.insert(first_coordinate.clone(), "FP".to_string());
        Self::new(
            FontOptions {
                font: crate::Font::EguiMonospace,
                background_is_transparent: true,
                font_height: 18.,
            },
            true,
            overlay,
            "Example Title",
        )
        .expect("Failed to generate example")
    }

    fn get_overlays(&self) -> impl Iterator<Item = (&CoordinatePoint, &BitMapText)> {
        self.overlay_indices
            .iter()
            .map(|(k, i)| (k, &self.overlay_bitmaps[*i]))
    }
}
/// A representation of a bitmap with overlay text
pub struct Data<Color> {
    /// width of bitmap in pixels
    pub width: usize,
    /// height of bitmap in pixels
    pub height: usize,
    /// Colors for each pixel, row by row
    pub data: Vec<Color>,
    /// the first-data point (row 0, column 0) in user-given coordinates
    pub first_point_coordinate: CoordinatePoint,
    /// overlay text
    pub overlay: Overlay,
}
impl<Color: Clone> Data<Color> {
    fn lookup(&self, point: &CoordinatePoint) -> Option<Color> {
        //let offset = point-self.first_point_coordinate;
        if point.x < self.first_point_coordinate.x
            || point.y < self.first_point_coordinate.y
            || (point.x - self.first_point_coordinate.x) as usize >= self.width
            || (point.y - self.first_point_coordinate.y) as usize >= self.height
        {
            None
        } else {
            let CoordinateVec { x, y } = point - &self.first_point_coordinate;
            Some(self.data[x + y * self.width].clone())
        }
    }

    fn bounding_box(&self) -> CoordinateRect {
        let left_top = self.first_point_coordinate.clone();
        let right_bottom = &left_top
            + CoordinateVec {
                x: self.width,
                y: self.height,
            };
        CoordinateRect {
            left_top,
            right_bottom,
        }
    }
}
impl Data<egui::Color32> {
    /// Generate an example data set
    pub fn example(width: usize, height: usize, first_point_coordinate: CoordinatePoint) -> Self {
        let mut data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let c = crate::colors::convert_from_oklab(oklab::Oklab {
                    l: 0.8,
                    a: 2. * x as f32 / (width - 1) as f32 - 1.,
                    b: 2. * y as f32 / (height - 1) as f32 - 1.,
                });
                data.push(c);
            }
        }
        let font = FontOptions {
            font: crate::Font::EguiMonospace,
            background_is_transparent: true,
            font_height: 12.,
        };
        let mut overlay_text = std::collections::HashMap::default();
        overlay_text.insert(first_point_coordinate.clone(), "FP".to_string());
        Self {
            width,
            height,
            data,
            first_point_coordinate,
            overlay: Overlay::new(font, true, overlay_text, "Test")
                .expect("Failed to generate overlay"),
        }
    }
    /// Generate an example data set
    pub fn example_circle(width: usize, height: usize, center: CoordinatePoint) -> Self {
        let mut data = Vec::new();
        let mut overlay_text = std::collections::HashMap::default();
        let font = FontOptions {
            font: crate::Font::EguiMonospace,
            background_is_transparent: true,
            font_height: 12.,
        };
        for y in 0..height {
            for x in 0..width {
                let distance_squared = (center.x - x as i32).pow(2) + (center.y - y as i32).pow(2);
                let max_squared = ((width + height) / 2).pow(2) as i32;
                let b = distance_squared as f32 / max_squared as f32;
                let b = if b < 1. { b } else { 1. };
                let b = b * 2. - 1.;
                let c = crate::colors::convert_from_oklab(oklab::Oklab { l: 0.8, a: 0., b });
                data.push(c);
                overlay_text.insert(
                    CoordinatePoint {
                        x: x as i32,
                        y: y as i32,
                    },
                    format!("{x}|{y}"),
                );
            }
        }

        Self {
            width,
            height,
            data,
            first_point_coordinate: CoordinatePoint {
                x: center.x - width as i32 / 2,
                y: center.y - height as i32 / 2,
            },
            overlay: Overlay::new(font, true, overlay_text, "Test")
                .expect("Failed to render both title and fallback"),
        }
    }
}

/// This types bundles a color with a size
pub struct ColorWithThickness<Color> {
    /// Color of this item
    pub color: Color,
    /// Thickness in pixels
    pub thickness: usize,
}

pub(crate) struct DataWithMetadata<Key, Color> {
    pub key: Key,
    pub data: Data<Color>,
}

pub(crate) struct ShowMultiMap<Key, Color> {
    data: Vec<DataWithMetadata<Key, Color>>,
    boundary_between_data: ColorWithThickness<Color>,
    colorbar: Option<(crate::colors::Gradient<Color>, usize, (f32, f32))>,
    background: Color,
    boundary_unselected: ColorWithThickness<Color>,
    boundary_selected: Color,
    boundary_factor_min: usize,
    drag_area: Option<((CoordinatePoint, CoordinatePoint), CoordinatePoint)>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum RenderProblem {
    CountIsZero,
    WidthSmallerThanColorBar,
    NoData,
    ClipboardIssue(String),
}

pub(crate) struct ShowMultiMapSettings<Color> {
    pub boundary_between_data: ColorWithThickness<Color>,
    pub colorbar: Option<(crate::colors::Gradient<Color>, usize, (f32, f32))>,
    pub background: Color,
    pub boundary_unselected: ColorWithThickness<Color>,
    pub boundary_selected: Color,
    pub boundary_factor_min: usize,
}

impl<Key: std::hash::Hash + Eq + Clone, Color: Clone + GammyMultiplyable + BitMapDrawable>
    ShowMultiMap<Key, Color>
{
    pub(crate) fn default_state(&self) -> MultimapState<Key> {
        let to_plot = self.data.iter().map(|d| (d.key.clone(), true)).collect();

        MultimapState {
            selected: Default::default(),
            shown_rectangle: None,
            to_plot,
        }
    }
    pub(crate) fn with_settings(
        data: Vec<DataWithMetadata<Key, Color>>,
        settings: ShowMultiMapSettings<Color>,
    ) -> Self {
        let ShowMultiMapSettings {
            boundary_between_data,
            colorbar,
            background,
            boundary_unselected,
            boundary_selected,
            boundary_factor_min,
        } = settings;
        Self {
            data,
            boundary_between_data,
            colorbar,
            background,
            boundary_unselected,
            boundary_selected,
            boundary_factor_min,
            drag_area: Default::default(),
        }
    }
    pub(crate) fn render(
        &self,
        width: usize,
        height: usize,
        state: &mut MultimapState<Key>,
    ) -> Result<Vec<Color>, RenderProblem> {
        if state.shown_rectangle.is_none() {
            if self.data.is_empty() {
                return Err(RenderProblem::NoData);
            } else {
                state.shown_rectangle = Some(home_rect(&self.data, &state.to_plot));
            }
        }
        let shown_rectangle = state.shown_rectangle.as_ref().unwrap();

        let mut data_sets = self
            .data
            .iter()
            .filter_map(|d| {
                if state.to_plot(&d.key) {
                    Some(&d.data)
                } else {
                    None
                }
            })
            .rev()
            .collect::<Vec<_>>();
        let count = data_sets.len();

        if count == 0 {
            return Err(RenderProblem::CountIsZero);
        }
        let (data_columns, data_rows) = compute_columns_rows(count);
        assert!(data_columns > 0);
        assert!(data_rows > 0);
        let (width_per_data, height_per_data) = {
            let cb_thickness = self
                .colorbar
                .as_ref()
                .map(|(_, thickness, _)| thickness + self.boundary_between_data.thickness)
                .unwrap_or(0);
            let width_without_colorbar = if width >= cb_thickness {
                width - cb_thickness
            } else {
                return Err(RenderProblem::WidthSmallerThanColorBar);
            };
            let width_without_colorbar_and_boundaries =
                width_without_colorbar - self.boundary_between_data.thickness * (data_columns - 1);
            let width_per_data = width_without_colorbar_and_boundaries / data_columns;
            let height_without_colorbar_and_boundaries =
                height - self.boundary_between_data.thickness * (data_rows - 1);
            let height_per_data = height_without_colorbar_and_boundaries / data_rows;
            (width_per_data, height_per_data)
        };
        let plot_width = data_columns * width_per_data
            + self.boundary_between_data.thickness * (data_columns - 1);
        let mut rendered = vec![self.background.clone(); width * height];
        let render_width = width;
        fn draw_axis_label<Color: BitMapDrawable + Clone>(
            data: &mut [Color],
            bitmapfont: &BitMapText,
            x_offset: usize,
            y_offset: usize,
            render_width: usize,
            background_is_transparent: bool,
            background: &Color,
        ) {
            for column in 0..bitmapfont.width {
                for row in 0..bitmapfont.height {
                    let x = column as usize + x_offset;
                    let y = row as usize + y_offset;
                    let i = x + y * render_width;
                    let c = match (background_is_transparent, bitmapfont.fetch(column, row)) {
                        (true, None) => {
                            /* nothing to do - but this should never occur*/
                            continue;
                        }
                        (false, None) => background.clone(),

                        (true, Some(gray)) => {
                            if let Some(c) = data.get(i) {
                                c.saturating_add(gray)
                            } else {
                                continue;
                            }
                        }
                        (false, Some(gray)) => Color::gray(gray),
                    };
                    data[i] = c;
                }
            }
        }

        for data_row in 0..data_rows {
            // add boundary rows above the data to draw in this iteration
            if data_row > 0 {
                for i in 0..self.boundary_between_data.thickness {
                    let row = data_row * (height_per_data + self.boundary_between_data.thickness)
                        + i
                        - self.boundary_between_data.thickness;
                    for column in 0..plot_width {
                        rendered[column + row * width] = self.boundary_between_data.color.clone();
                    }
                }
            }
            for data_column in 0..data_columns {
                // add boundary columns to the left of the data to draw in this iteration
                if data_column > 0 {
                    for i in 0..height_per_data {
                        let row =
                            data_row * (height_per_data + self.boundary_between_data.thickness) + i;
                        for j in 0..self.boundary_between_data.thickness {
                            let column = j + data_column
                                * (width_per_data + self.boundary_between_data.thickness)
                                - self.boundary_between_data.thickness;
                            rendered[column + row * width] =
                                self.boundary_between_data.color.clone();
                        }
                    }
                }
                // render data
                if let Some(data) = data_sets.pop() {
                    let shown_rectangle = shown_rectangle - &CoordinatePoint { x: 0, y: 0 };
                    let delta = shown_rectangle.delta();
                    let width_per_point = width_per_data / delta.x;
                    let height_per_point = height_per_data / delta.y;
                    let overlay_offset_lt = if width_per_point > 0 && height_per_point > 0 {
                        let boundary_thickness = if width_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                            && height_per_point
                                > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        };
                        let offset_x = (width_per_data.rem_euclid(width_per_point) + 1) / 2;
                        let offset_y = (height_per_data.rem_euclid(height_per_point) + 1) / 2;
                        for row in 0..height_per_data {
                            for column in 0..width_per_data {
                                let render_point = {
                                    let mut is_boundary = false;
                                    let x = if column < offset_x {
                                        if column + boundary_thickness >= offset_x {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.x - 1
                                    } else {
                                        let column = column - offset_x;
                                        let x = column / width_per_point;
                                        let rem = column.rem_euclid(width_per_point);
                                        if rem < boundary_thickness
                                            || rem + boundary_thickness >= width_per_point
                                        {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.x + x as i32
                                    };
                                    let y = if row < offset_y {
                                        if row + boundary_thickness >= offset_y {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.y - 1
                                    } else {
                                        let row = row - offset_y;
                                        let y = row / height_per_point;
                                        let rem = row.rem_euclid(height_per_point);
                                        if rem < boundary_thickness
                                            || rem + boundary_thickness >= height_per_point
                                        {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.y + y as i32
                                    };
                                    RenderPoint {
                                        coordinate: CoordinatePoint { x, y },
                                        is_boundary,
                                    }
                                };
                                self.update_color(
                                    data,
                                    render_point,
                                    row,
                                    data_row,
                                    height_per_data,
                                    column,
                                    data_column,
                                    width_per_data,
                                    &mut rendered,
                                    width,
                                    state,
                                );
                            }
                        }
                        Some((offset_x, offset_y))
                    } else if width_per_point > 0 && height_per_point == 0 {
                        let boundary_thickness = if width_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        };
                        let offset_x = (width_per_data.rem_euclid(width_per_point) + 1) / 2;
                        for row in 0..height_per_data {
                            for column in 0..width_per_data {
                                let render_point = {
                                    let mut is_boundary = false;
                                    let x = if column < offset_x {
                                        if column + boundary_thickness >= offset_x {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.x - 1
                                    } else {
                                        let column = column - offset_x;
                                        let x = column / width_per_point;
                                        let rem = column.rem_euclid(width_per_point);
                                        if rem < boundary_thickness
                                            || rem + boundary_thickness >= width_per_point
                                        {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.x + x as i32
                                    };
                                    let y = row * delta.y / height_per_data;
                                    let y = shown_rectangle.left_top.y + y as i32;
                                    RenderPoint {
                                        coordinate: CoordinatePoint { x, y },
                                        is_boundary,
                                    }
                                };
                                self.update_color(
                                    data,
                                    render_point,
                                    row,
                                    data_row,
                                    height_per_data,
                                    column,
                                    data_column,
                                    width_per_data,
                                    &mut rendered,
                                    width,
                                    state,
                                );
                            }
                        }
                        None
                    } else if width_per_point == 0 && height_per_point > 0 {
                        let boundary_thickness = if height_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        };
                        let offset_y = (height_per_data.rem_euclid(height_per_point) + 1) / 2;
                        for row in 0..height_per_data {
                            for column in 0..width_per_data {
                                let render_point = {
                                    let mut is_boundary = false;
                                    let x = column * delta.x / width_per_data;
                                    let x = shown_rectangle.left_top.x + x as i32;
                                    let y = if row < offset_y {
                                        if row + boundary_thickness >= offset_y {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.y - 1
                                    } else {
                                        let row = row - offset_y;
                                        let y = row / height_per_point;
                                        let rem = row.rem_euclid(height_per_point);
                                        if rem < boundary_thickness
                                            || rem + boundary_thickness >= height_per_point
                                        {
                                            is_boundary = true;
                                        }
                                        shown_rectangle.left_top.y + y as i32
                                    };
                                    RenderPoint {
                                        coordinate: CoordinatePoint { x, y },
                                        is_boundary,
                                    }
                                };
                                self.update_color(
                                    data,
                                    render_point,
                                    row,
                                    data_row,
                                    height_per_data,
                                    column,
                                    data_column,
                                    width_per_data,
                                    &mut rendered,
                                    width,
                                    state,
                                );
                            }
                        }
                        None
                    } else {
                        for row in 0..height_per_data {
                            for column in 0..width_per_data {
                                let render_point = {
                                    let x = column * delta.x / width_per_data;
                                    let y = row * delta.y / height_per_data;
                                    let offset = CoordinateVec { x, y };
                                    let point = &shown_rectangle.left_top + offset;
                                    RenderPoint {
                                        coordinate: point,
                                        is_boundary: false,
                                    }
                                };
                                self.update_color(
                                    data,
                                    render_point,
                                    row,
                                    data_row,
                                    height_per_data,
                                    column,
                                    data_column,
                                    width_per_data,
                                    &mut rendered,
                                    width,
                                    state,
                                );
                            }
                        }
                        None
                    }; // add title
                    {
                        let title = &data.overlay.title;
                        let mut font = data.overlay.font.clone();
                        let mut title_to_draw = None;
                        while font.font_height > 8. {
                            if let Some(title) = font.render(title) {
                                if (title.width as usize) < (width_per_data * 8 / 10) {
                                    title_to_draw = Some(title);
                                    break;
                                }
                            }
                            font.font_height -= 1.0;
                        }
                        if let Some(title) = title_to_draw {
                            draw_axis_label(
                                &mut rendered,
                                &title,
                                data_column
                                    * (width_per_data + self.boundary_between_data.thickness)
                                    + (width_per_data.saturating_sub(title.width as usize)) / 2,
                                data_row * (height_per_data + self.boundary_between_data.thickness),
                                render_width,
                                data.overlay.font.background_is_transparent,
                                &self.background,
                            );
                        }
                    }
                    // add overlays
                    if let Some((ox, oy)) = overlay_offset_lt {
                        for (pos, bitmap) in data.overlay.get_overlays() {
                            if pos.x >= shown_rectangle.left_top.x
                                && pos.y >= shown_rectangle.left_top.y
                                && pos.x < shown_rectangle.right_bottom.x
                                && pos.y < shown_rectangle.right_bottom.y
                                && bitmap.width as usize <= width_per_point
                                && bitmap.height as usize <= height_per_point
                            {
                                let dx = (pos.x - shown_rectangle.left_top.x) as usize;
                                let dy = (pos.y - shown_rectangle.left_top.y) as usize;
                                draw_axis_label(
                                    &mut rendered,
                                    bitmap,
                                    data_column
                                        * (width_per_data + self.boundary_between_data.thickness)
                                        + ox
                                        + dx * width_per_point
                                        + width_per_point.saturating_sub(bitmap.width as usize) / 2,
                                    data_row
                                        * (height_per_data + self.boundary_between_data.thickness)
                                        + oy
                                        + dy * height_per_point
                                        + height_per_point.saturating_sub(bitmap.height as usize)
                                            / 2,
                                    render_width,
                                    data.overlay.font.background_is_transparent,
                                    &self.background,
                                );
                            }
                        }
                    }
                    // add corners
                    if data.overlay.show_coordinates {
                        let ShowRect {
                            left_top: ShowPoint { x: ltx, y: lty },
                            right_bottom: ShowPoint { x: rbx, y: rby },
                        } = state.shown_rectangle.clone().unwrap_or_default();
                        let rbx = rbx - 1;
                        let rby = rby - 1;
                        let lt = data.overlay.font.render(&format!("{ltx}|{lty}"));
                        let lb = data.overlay.font.render(&format!("{ltx}|{rby}"));
                        let rt = data.overlay.font.render(&format!("{rbx}|{lty}"));
                        let rb = data.overlay.font.render(&format!("{rbx}|{rby}"));
                        let lt = lt.map(|x| ((0, 0), x));
                        let lb: Option<((usize, usize), BitMapText)> = lb.map(|x: BitMapText| {
                            ((0, height_per_data.saturating_sub(x.height as usize)), x)
                        });
                        let rt = rt.map(|x: BitMapText| {
                            ((width_per_data.saturating_sub(x.width as usize), 0), x)
                        });
                        let rb = rb.map(|x: BitMapText| {
                            (
                                (
                                    width_per_data.saturating_sub(x.width as usize),
                                    height_per_data.saturating_sub(x.height as usize),
                                ),
                                x,
                            )
                        });
                        for ((dx, dy), font) in [lt, lb, rt, rb].into_iter().flatten() {
                            draw_axis_label(
                                &mut rendered,
                                &font,
                                data_column
                                    * (width_per_data + self.boundary_between_data.thickness)
                                    + dx,
                                data_row * (height_per_data + self.boundary_between_data.thickness)
                                    + dy,
                                render_width,
                                data.overlay.font.background_is_transparent,
                                &self.background,
                            );
                        }
                    }
                }
            }
        }

        // add colorbar
        if let Some((gradient, thickness, (lower, upper))) = &self.colorbar {
            let thickness = *thickness;
            for row in 0..height {
                for column in 0..self.boundary_between_data.thickness {
                    let column = width - self.boundary_between_data.thickness - thickness + column;
                    rendered[column + row * width] = self.boundary_between_data.color.clone();
                }
            }
            for row in 0..height {
                for column in 0..thickness {
                    let column = width - thickness + column;
                    let c = gradient.element_at(height - 1 - row, height).remove_alpha();
                    rendered[column + row * width] = c;
                }
            }
            if let Some(font) = self.data.first().map(|d| &d.data.overlay.font) {
                fn string_representation(value: f32, precision: usize) -> String {
                    let mut num = format!("{value:+3.precision$E}");
                    let exp = num.split_off(num.find('E').unwrap());
                    let (sign, exp) = if let Some(stripped) = exp.strip_prefix("E-") {
                        ('-', stripped)
                    } else {
                        ('+', &exp[1..])
                    };
                    num.push_str(&format!("E{}{:0>pad$}", sign, exp, pad = 2));
                    num
                }
                let count = 5; //TODO: make this configurable
                let count = std::cmp::max(2, count);
                for (i, f) in (0..count)
                    .map(|i| lower + (upper - lower) / (count as f32 - 1.) * (i as f32))
                    .rev()
                    .enumerate()
                {
                    let mut bitmapfont = None;
                    let mut font = font.clone();
                    'outer: while font.font_height > 8. {
                        for max_precision in (1..5).rev() {
                            let s = string_representation(f, max_precision);
                            if let Some(font) = BitMapText::new(&s, &font) {
                                if font.width < thickness as i32 {
                                    bitmapfont = Some(font);
                                    break 'outer;
                                }
                            }
                        }
                        font.font_height -= 1.;
                    }
                    let f = if let Some(bitmapfont) = bitmapfont {
                        bitmapfont
                    } else {
                        continue;
                    };
                    let target_center = (height * i / (count - 1)) as i32;
                    let top = target_center - f.height / 2;
                    if height as i32 > f.height && width as i32 > f.width {
                        let top = top.clamp(0, height as i32 - f.height) as usize;
                        let left = std::cmp::max(0, width as i32 - f.width) as usize;
                        draw_axis_label(
                            &mut rendered,
                            &f,
                            left,
                            top,
                            render_width,
                            font.background_is_transparent,
                            &self.background,
                        );
                    }
                }
            }
        }
        Ok(rendered)
    }

    #[allow(clippy::too_many_arguments)]
    fn update_color(
        &self,
        data: &Data<Color>,
        RenderPoint {
            coordinate,
            is_boundary,
        }: RenderPoint,
        row: usize,
        data_row: usize,
        height_per_data: usize,
        column: usize,
        data_column: usize,
        width_per_data: usize,
        rendered: &mut [Color],
        width: usize,
        state: &MultimapState<Key>,
    ) {
        let c = if let Some(c) = data.lookup(&coordinate) {
            if is_boundary {
                if state.selected.contains(&coordinate) {
                    self.boundary_selected.clone()
                } else {
                    self.boundary_unselected.color.clone()
                }
            } else {
                c
            }
        } else {
            self.background.clone()
        };
        let c = if let Some(((lt, rb), _)) = &self.drag_area {
            if lt.x <= coordinate.x
                && lt.y <= coordinate.y
                && coordinate.x <= rb.x
                && coordinate.y <= rb.y
            {
                c.gamma_multiply(0.5)
            } else {
                c
            }
        } else {
            c
        };
        let c = c.remove_alpha();
        let row = row + data_row * (height_per_data + self.boundary_between_data.thickness);
        let column = column + data_column * (width_per_data + self.boundary_between_data.thickness);
        rendered[column + row * width] = c;
    }

    pub(crate) fn convert_multimap2bitmap(
        &self,
        MultiMapPoint { x: column, y: row }: MultiMapPoint,
        [width, height]: [usize; 2],
        state: &MultimapState<Key>,
    ) -> crate::MultiMapPosition<Key>
    where
        Key: Clone,
    {
        let data_sets = self
            .data
            .iter()
            .filter_map(|DataWithMetadata { key, data }| {
                if state.to_plot(key) {
                    Some((key, data))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let count = data_sets.len();
        if count == 0 {
            return crate::MultiMapPosition::NotHovering;
        }
        let (data_columns, data_rows) = compute_columns_rows(count);
        assert!(data_columns > 0);
        assert!(data_rows > 0);
        let (width_per_data, height_per_data) = {
            let cb_thickness = self
                .colorbar
                .as_ref()
                .map(|(_, thickness, _)| *thickness + self.boundary_between_data.thickness)
                .unwrap_or(0);
            let width_without_colorbar = if width >= cb_thickness {
                width - cb_thickness
            } else {
                return crate::MultiMapPosition::NotHovering;
            };
            let width_without_colorbar_and_boundaries =
                width_without_colorbar - self.boundary_between_data.thickness * (data_columns - 1);
            let width_per_data = width_without_colorbar_and_boundaries / data_columns;
            let height_without_colorbar_and_boundaries =
                height - self.boundary_between_data.thickness * (data_rows - 1);
            let height_per_data = height_without_colorbar_and_boundaries / data_rows;
            (width_per_data, height_per_data)
        };
        let data_column = column / width_per_data;
        let data_row = row / height_per_data;
        let data_index = data_row * data_columns + data_column;
        let plot_width = data_columns * width_per_data
            + self.boundary_between_data.thickness * (data_columns - 1);
        if column < plot_width {
            if let Some((key, data)) = data_sets.get(data_index) {
                let shown_rectangle = &state.shown_rectangle.clone().unwrap_or_default()
                    - &CoordinatePoint { x: 0, y: 0 };
                let delta = shown_rectangle.delta();
                let width_per_point = width_per_data / delta.x;
                let height_per_point = height_per_data / delta.y;
                let row = row % height_per_data;
                let column = column % width_per_data;
                let render_point = if width_per_point > 0 && height_per_point > 0 {
                    let boundary_thickness = {
                        if width_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                            && height_per_point
                                > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        }
                    };
                    let offset_x = (width_per_data.rem_euclid(width_per_point) + 1) / 2;
                    let offset_y = (height_per_data.rem_euclid(height_per_point) + 1) / 2;
                    let mut is_boundary = false;
                    let x = if column < offset_x {
                        if column + boundary_thickness >= offset_x {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.x - 1
                    } else {
                        let column = column - offset_x;
                        let x = column / width_per_point;
                        let rem = column.rem_euclid(width_per_point);
                        if rem < boundary_thickness || rem + boundary_thickness >= width_per_point {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.x + x as i32
                    };
                    let y = if row < offset_y {
                        if row + boundary_thickness >= offset_y {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.y - 1
                    } else {
                        let row = row - offset_y;
                        let y = row / height_per_point;
                        let rem = row.rem_euclid(height_per_point);
                        if rem < boundary_thickness || rem + boundary_thickness >= height_per_point
                        {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.y + y as i32
                    };
                    RenderPoint {
                        coordinate: CoordinatePoint { x, y },
                        is_boundary,
                    }
                } else if width_per_point > 0 && height_per_point == 0 {
                    let boundary_thickness = {
                        if width_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        }
                    };
                    let offset_x = (width_per_data.rem_euclid(width_per_point) + 1) / 2;
                    let mut is_boundary = false;
                    let x = if column < offset_x {
                        if column + boundary_thickness >= offset_x {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.x - 1
                    } else {
                        let column = column - offset_x;
                        let x = column / width_per_point;
                        let rem = column.rem_euclid(width_per_point);
                        if rem < boundary_thickness || rem + boundary_thickness >= width_per_point {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.x + x as i32
                    };
                    let y = row * delta.y / height_per_data;
                    let y = shown_rectangle.left_top.y + y as i32;
                    RenderPoint {
                        coordinate: CoordinatePoint { x, y },
                        is_boundary,
                    }
                } else if width_per_point == 0 && height_per_point > 0 {
                    let boundary_thickness = {
                        if height_per_point
                            > self.boundary_factor_min * self.boundary_unselected.thickness
                        {
                            self.boundary_unselected.thickness
                        } else {
                            0
                        }
                    };
                    let offset_y = (height_per_data.rem_euclid(height_per_point) + 1) / 2;

                    let mut is_boundary = false;
                    let x = column * delta.x / width_per_data;
                    let x = shown_rectangle.left_top.x + x as i32;
                    let y = if row < offset_y {
                        if row + boundary_thickness >= offset_y {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.y - 1
                    } else {
                        let row = row - offset_y;
                        let y = row / height_per_point;
                        let rem = row.rem_euclid(height_per_point);
                        if rem < boundary_thickness || rem + boundary_thickness >= height_per_point
                        {
                            is_boundary = true;
                        }
                        shown_rectangle.left_top.y + y as i32
                    };
                    RenderPoint {
                        coordinate: CoordinatePoint { x, y },
                        is_boundary,
                    }
                } else {
                    let x = column * delta.x / width_per_data;
                    let y = row * delta.y / height_per_data;
                    let offset = CoordinateVec { x, y };
                    let point = &shown_rectangle.left_top + offset;
                    RenderPoint {
                        coordinate: point,
                        is_boundary: false,
                    }
                };
                let RenderPoint {
                    coordinate,
                    is_boundary: _,
                } = render_point;
                let key: &Key = key;
                let key: Key = key.clone();
                if data.lookup(&coordinate).is_some() {
                    crate::MultiMapPosition::Pixel(key, coordinate)
                } else {
                    crate::MultiMapPosition::NoData(key, coordinate)
                }
            } else {
                crate::MultiMapPosition::NotHovering
            }
        } else if let Some((g, thickness, (lower, upper))) = &self.colorbar {
            if column + thickness >= width {
                let relative_distance = (row as f32) / (height as f32); // this is a number between 0 and 1
                let f = g.fetch_value(*lower, *upper, 1. - relative_distance);
                crate::MultiMapPosition::Colorbar(f)
            } else {
                crate::MultiMapPosition::NotHovering
            }
        } else {
            crate::MultiMapPosition::NotHovering
        }
    }

    pub(crate) fn zoom(&mut self, zoom_increment: i32, shown_rectangle: &mut ShowRect) {
        if zoom_increment < 0
            || (shown_rectangle.right_bottom.x - shown_rectangle.left_top.x
                > 3 + zoom_increment * 2)
        {
            shown_rectangle.left_top.x += zoom_increment;
            shown_rectangle.right_bottom.x -= zoom_increment;
        }
        if zoom_increment < 0
            || (shown_rectangle.right_bottom.y - shown_rectangle.left_top.y
                > 3 + zoom_increment * 2)
        {
            shown_rectangle.left_top.y += zoom_increment;
            shown_rectangle.right_bottom.y -= zoom_increment;
        }
    }

    pub(crate) fn translate_keyboard(
        &mut self,
        direction: KeyBoardDirection,
        shown_rectangle: &mut ShowRect,
    ) {
        let (dx, dy) = match direction {
            KeyBoardDirection::Up => (0, -1),
            KeyBoardDirection::Down => (0, 1),
            KeyBoardDirection::Left => (-1, 0),
            KeyBoardDirection::Right => (1, 0),
        };
        let delta = CoordinatePoint { x: dx, y: dy };
        self.translate(delta, shown_rectangle);
    }
    pub fn translate(&mut self, delta: CoordinatePoint, shown_rectangle: &mut ShowRect) {
        shown_rectangle.left_top.x += delta.x;
        shown_rectangle.left_top.y += delta.y;
        shown_rectangle.right_bottom.x += delta.x;
        shown_rectangle.right_bottom.y += delta.y;
    }

    pub fn center_to(&mut self, pos: &CoordinatePoint, shown_rectangle: &mut ShowRect) {
        let dx = shown_rectangle.right_bottom.x - shown_rectangle.left_top.x;
        let dy = shown_rectangle.right_bottom.y - shown_rectangle.left_top.y;
        shown_rectangle.left_top.x = pos.x - (dx - dx / 2);
        shown_rectangle.left_top.y = pos.y - (dy - dy / 2);
        shown_rectangle.right_bottom.x = pos.x + dx / 2;
        shown_rectangle.right_bottom.y = pos.y + dy / 2;
    }

    pub fn select(
        &mut self,
        pos: &CoordinatePoint,
        ctrl_is_pressed: bool,
        selected: &mut std::collections::HashSet<CoordinatePoint>,
    ) {
        let was_selected_before = selected.remove(pos);
        if !ctrl_is_pressed {
            selected.clear();
        }
        if !was_selected_before {
            selected.insert(pos.clone());
        }
    }

    pub fn drag_start(&mut self, pos: &CoordinatePoint) {
        self.drag_area = Some(((pos.clone(), pos.clone()), pos.clone()));
    }

    pub fn drag_is_ongoing(&mut self, pos: &CoordinatePoint) -> bool {
        if let Some((before, start)) = self.drag_area.take() {
            let lt = CoordinatePoint {
                x: std::cmp::min(start.x, pos.x),
                y: std::cmp::min(start.y, pos.y),
            };
            let rb = CoordinatePoint {
                x: std::cmp::max(start.x, pos.x),
                y: std::cmp::max(start.y, pos.y),
            };
            let unchanged = before.0 == lt && before.1 == rb;
            self.drag_area = Some(((lt, rb), start));
            !unchanged
        } else {
            false
        }
    }

    pub fn drag_release(&mut self, pos: Option<&CoordinatePoint>, shown_rectangle: &mut ShowRect) {
        if let (Some((_, CoordinatePoint { x: ax, y: ay })), Some(pos)) =
            (self.drag_area.take(), pos)
        {
            let bx = pos.x;
            let by = pos.y;
            let lt = ShowPoint {
                x: std::cmp::min(ax, bx),
                y: std::cmp::min(ay, by),
            };
            let rb = ShowPoint {
                x: std::cmp::max(ax, bx) + 1,
                y: std::cmp::max(ay, by) + 1,
            };
            // check that at least three dies are selected
            let dx = rb.x - lt.x;
            let dy = rb.y - lt.y;
            if dx > 3 + 1 && dy > 3 + 1 {
                shown_rectangle.left_top = lt;
                shown_rectangle.right_bottom = rb;
            }
        }
    }

    pub(crate) fn home(&self, state: &mut MultimapState<Key>) {
        state.shown_rectangle = Some(home_rect(&self.data, &state.to_plot));
    }
}

pub(crate) fn home_rect<Key: std::hash::Hash + Eq, Color: Clone>(
    data: &[DataWithMetadata<Key, Color>],
    to_plot: &std::collections::HashMap<Key, bool>,
) -> ShowRect {
    let bounding_boxes = data
        .iter()
        .filter(|d| to_plot.get(&d.key).cloned().unwrap_or(true))
        .map(|d| d.data.bounding_box())
        .collect::<Vec<_>>();
    let lt_x = bounding_boxes
        .iter()
        .map(|b| b.left_top.x)
        .min()
        .unwrap_or(0);
    let lt_y = bounding_boxes
        .iter()
        .map(|b| b.left_top.y)
        .min()
        .unwrap_or(0);
    let rb_x = bounding_boxes
        .iter()
        .map(|b| b.right_bottom.x)
        .max()
        .unwrap_or(1);
    let rb_y = bounding_boxes
        .iter()
        .map(|b| b.right_bottom.y)
        .max()
        .unwrap_or(1);
    ShowRect {
        left_top: ShowPoint { x: lt_x, y: lt_y },
        right_bottom: ShowPoint { x: rb_x, y: rb_y },
    }
}

#[test]
fn render_simple_tests() {
    fn dummy_data() -> ShowMultiMap<usize, char> {
        let data = vec![
            Data {
                width: 5,
                height: 5,
                data: (0..25)
                    .map(|x| (x % 10).to_string().chars().next().unwrap())
                    .collect(),
                first_point_coordinate: CoordinatePoint { x: 0, y: 0 },
                overlay: Overlay::example(&CoordinatePoint { x: 1, y: 1 }),
            },
            Data {
                width: 5,
                height: 5,
                data: (0..25)
                    .map(|x| (x % 10).to_string().chars().next().unwrap())
                    .collect(),
                first_point_coordinate: CoordinatePoint { x: 1, y: 0 },
                overlay: Overlay::example(&CoordinatePoint { x: 1, y: 1 }),
            },
            Data {
                width: 5,
                height: 5,
                data: (0..25)
                    .map(|x| (x % 10).to_string().chars().next().unwrap())
                    .collect(),
                first_point_coordinate: CoordinatePoint { x: 0, y: 1 },
                overlay: Overlay::example(&CoordinatePoint { x: 1, y: 1 }),
            },
            Data {
                width: 5,
                height: 5,
                data: (0..25)
                    .map(|x| (x % 10).to_string().chars().next().unwrap())
                    .collect(),
                first_point_coordinate: CoordinatePoint { x: 1, y: 1 },
                overlay: Overlay::example(&CoordinatePoint { x: 1, y: 1 }),
            },
        ];
        ShowMultiMap {
            data: data
                .into_iter()
                .enumerate()
                .map(|(i, d)| DataWithMetadata { key: i, data: d })
                .collect(),
            boundary_between_data: ColorWithThickness {
                color: '-',
                thickness: 2,
            },
            colorbar: Some((crate::colors::Gradient(vec!['a', 'b', 'c']), 4, (0., 1.))),
            background: '.',
            boundary_unselected: ColorWithThickness {
                color: 'r',
                thickness: 1,
            },
            boundary_selected: 'w',
            boundary_factor_min: 7,
            drag_area: None,
        }
    }
    let width = 66;
    let height = 23;
    let mut state = dummy_data().default_state();
    let rendered = dummy_data().render(width, height, &mut state).unwrap();
    dbg!((width, height));
    for (i, line) in rendered
        .chunks(width)
        .map(|x| x.iter().collect::<String>())
        .enumerate()
    {
        println!("{i:03},{line}");
    }
}
#[test]
fn render_simple_tests2() {
    fn dummy_data() -> ShowMultiMap<usize, char> {
        let data = vec![Data {
            width: 9,
            height: 6,
            data: (0..9 * 6)
                .map(|x| (x % 10).to_string().chars().next().unwrap())
                .collect(),
            first_point_coordinate: CoordinatePoint { x: -1, y: -1 },
            overlay: Overlay::example(&CoordinatePoint { x: 1, y: 1 }),
        }];
        ShowMultiMap {
            data: data
                .into_iter()
                .enumerate()
                .map(|(i, d)| DataWithMetadata { key: i, data: d })
                .collect(),
            boundary_between_data: ColorWithThickness {
                color: '-',
                thickness: 2,
            },
            colorbar: Some((crate::colors::Gradient(vec!['a', 'b', 'c']), 4, (0., 1.))),
            background: '.',
            boundary_unselected: ColorWithThickness {
                color: 'r',
                thickness: 1,
            },
            boundary_selected: 'w',
            boundary_factor_min: 3,
            drag_area: None,
        }
    }
    let width = 66;
    let height = 23;
    let mut state = dummy_data().default_state();
    let rendered = dummy_data().render(width, height, &mut state).unwrap();
    dbg!((width, height));
    for (i, line) in rendered
        .chunks(width)
        .map(|x| x.iter().collect::<String>())
        .enumerate()
    {
        println!("{i:03},{line}");
    }
}

#[test]
fn compute_columns_rows_test() {
    for (i, a) in [
        (0, (0, 0)),
        (1, (1, 1)),
        (2, (2, 1)),
        (3, (2, 2)),
        (4, (2, 2)),
        (5, (3, 2)),
        (6, (3, 2)),
        (7, (3, 3)),
        (8, (3, 3)),
        (9, (3, 3)),
        (10, (4, 3)),
        (11, (4, 3)),
        (12, (4, 3)),
        (13, (4, 4)),
        (14, (4, 4)),
        (15, (4, 4)),
        (16, (4, 4)),
        (17, (5, 4)),
    ] {
        assert_eq!(a, compute_columns_rows(i));
    }
}
fn compute_columns_rows(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let data_columns = (count as f64).sqrt().ceil() as usize;
    let mut data_rows = count / data_columns;
    while data_rows * data_columns < count {
        data_rows += 1;
    }
    (data_columns, data_rows)
}
