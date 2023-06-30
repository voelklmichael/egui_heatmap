use std::fmt::Debug;

use crate::multimap::KeyBoardDirection;
pub use crate::multimap::{
    BitMapText, ColorWithThickness, CoordinatePoint, CoordinateRect, Data, FontOptions, Overlay,
    RenderProblem,
};
use egui::Color32 as Color;
use egui_extras::RetainedImage as RenderedImage;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Localization {
    text_copy_to_clipboard_delayed: String, //"Copy to Clipboard in 3 seconds"
    text_copy_to_clipboard_instantly: String, //"Copy to Clipboard"
    text_hide: String,                      //"Hide"
    text_show_all: String,                  //"Show all"
    text_unselect_all: String,              //"Unselect all"
    text_home: String,                      //"Home"
}

impl Localization {
    fn english() -> Self {
        Self {
            text_copy_to_clipboard_delayed: "Copy to Clipboard in 3 seconds".to_string(),
            text_copy_to_clipboard_instantly: "Copy to Clipboard".to_string(),
            text_hide: "Hide".to_string(),
            text_show_all: "Show all".to_string(),
            text_unselect_all: "Unselect all".to_string(),
            text_home: "Home".to_string(),
        }
    }
}
/// This encodes the current state of the heatmap
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ShowState<Key: Eq + std::hash::Hash> {
    multimap: crate::multimap::MultimapState<Key>,
    localization: Localization,

    mouse: MultiMapPosition<Key>,
    clicked: bool, // Clicked plot can be fetched by mouse-value
    render_problem: Option<RenderProblem>,
    events: Vec<Event<Key>>,
}
/// Events which happend to the heatmap
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Event<Key> {
    /// A dataset shall be hidden
    Hide(Key),
    /// All datasets shall be shown
    ShowAll,
    /// All selected positions are cleared
    UnselectAll,
    /// The shown rectangle was changed
    /// The new rectanglel can be fetched via 'currently_showing'
    ShowRectangle,
    /// The selection was changed
    /// The new selection can be fetched via 'selected'
    Selection,
}
impl<Key: std::hash::Hash + Eq + Clone> ShowState<Key> {
    /// Select the given positions and only those
    pub fn make_selected(&mut self, selected: std::collections::HashSet<CoordinatePoint>) {
        self.multimap.selected = selected;
    }
    /// Clear selected positions
    pub fn clear_selected(&mut self) {
        self.multimap.selected.clear();
    }
    /// Get events
    pub fn events(&mut self) -> Vec<Event<Key>> {
        std::mem::take(&mut self.events)
    }
    /// Get the currently selected points
    pub fn selected(&self) -> &std::collections::HashSet<CoordinatePoint> {
        &self.multimap.selected
    }
    /// Fetch rectangle which is currently shown
    pub fn currently_showing(&self) -> Option<CoordinateRect> {
        self.multimap.currently_showing()
    }
    /// Check if there was an issue will rendering
    pub fn render_problem(&self) -> Option<&RenderProblem> {
        self.render_problem.as_ref()
    }
    /// Check if position was clicked
    pub fn clicked(&self) -> Option<&MultiMapPosition<Key>> {
        self.clicked.then_some(&self.mouse)
    }
    /// Check if position was clicked
    pub fn hover(&self) -> &MultiMapPosition<Key> {
        &self.mouse
    }

    fn has_hidden(&self) -> bool {
        self.multimap.to_plot.iter().any(|(_, &b)| !b)
    }

    fn can_hide(&self) -> bool {
        self.multimap.to_plot.iter().filter(|(_, &b)| b).count() > 1
    }

    fn hide(&mut self, key: &Key) {
        self.events.push(Event::Hide(key.clone()));
        if let Some(v) = self.multimap.to_plot.get_mut(key) {
            *v = false;
        } else {
            self.multimap.to_plot.insert(key.clone(), false);
        }
    }

    fn show_all(&mut self) {
        self.events.push(Event::ShowAll);
        self.multimap
            .to_plot
            .iter_mut()
            .for_each(|(_, p)| *p = true)
    }

    fn unselect_all(&mut self) -> bool {
        self.events.push(Event::UnselectAll);
        if self.multimap.selected.is_empty() {
            false
        } else {
            self.multimap.selected.clear();
            true
        }
    }

    fn change_rect(&mut self) -> &mut crate::multimap::ShowRect {
        self.multimap
            .shown_rectangle
            .as_mut()
            .expect("'Render' has to be called before this")
    }

    fn change_selected(&mut self) -> &mut std::collections::HashSet<CoordinatePoint> {
        self.events.push(Event::Selection);
        &mut self.multimap.selected
    }

    fn get_inner_mut(&mut self) -> &mut crate::multimap::MultimapState<Key> {
        &mut self.multimap
    }
}

/// Hover type
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum MultiMapPosition<Key> {
    /// Mouse is not hovering over widget
    NotHovering,
    /// Mouse is hovering over widget, but outside of data area
    NoData(Key, CoordinatePoint),
    /// Mouse is hovering over data area, containing the point in data coordinates
    Pixel(Key, CoordinatePoint),
    /// Mouse is over Colorbar
    Colorbar(f32),
}

impl<Key> MultiMapPosition<Key> {
    fn get_pos(&self) -> Option<&CoordinatePoint> {
        match self {
            MultiMapPosition::NotHovering => None,
            MultiMapPosition::NoData(_, pos) => Some(pos),
            MultiMapPosition::Pixel(_, pos) => Some(pos),
            MultiMapPosition::Colorbar(_) => None,
        }
    }

    fn get_key(&self) -> Option<&Key> {
        match self {
            MultiMapPosition::NotHovering => None,
            MultiMapPosition::NoData(key, _) => Some(key),
            MultiMapPosition::Pixel(key, _) => Some(key),
            MultiMapPosition::Colorbar(_) => None,
        }
    }
}

/// This is a bitmap widget, the main type of this crate
pub struct MultiBitmapWidget<Key> {
    showmap: crate::multimap::ShowMultiMap<Key, Color>,
    // size
    current_size: [f32; 2],
    dynamic_resizing: bool,
    // egui
    rendered_image: RenderedImage,
    debug_name: String,
    needs_rendering: bool,
    // interaction
    copy_to_clipboard_delay: Option<(std::time::Instant, [f32; 2])>,
    hide_key: Option<Key>,
}

/// This is the main settings type
pub struct MultiBitmapWidgetSettings {
    // egui
    /// Size of the render area.
    /// Use 'None' to request all available space
    pub start_size: Option<[f32; 2]>,
    /// id of this plot - needs to be locally unique (this is an egui-ID)
    pub id: String,
    // ShowMultiMapSettings
    /// Shall there be a boundary line between two data samples?
    pub boundary_between_data: ColorWithThickness<Color>,
    /// Shall there be a color bar?
    pub colorbar: Option<(crate::colors::Gradient<Color>, usize, (f32, f32))>,
    /// Background color
    pub background: Color,
    /// Boundary color for unselected points
    pub boundary_unselected: ColorWithThickness<Color>,
    /// Boundary color for selected points
    pub boundary_selected: Color,
    /// Minimimum ratio of pixels per point by boundary thickness to draw the boundary
    pub boundary_factor_min: usize,
}
const COPY_CLIPBOARD_DELAY: std::time::Duration = std::time::Duration::from_secs(3);

impl<Key: std::hash::Hash + Clone + Eq + Debug> MultiBitmapWidget<Key> {
    /// Get default state, in english
    pub fn default_state_english(&self) -> ShowState<Key> {
        ShowState {
            multimap: self.showmap.default_state(),
            localization: Localization::english(),
            mouse: MultiMapPosition::NotHovering,
            clicked: Default::default(),
            render_problem: Default::default(),
            events: Default::default(),
        }
    }
    /// Main Constructor. This assumes that the data coordinates are linearly and axis-aligned to the bitmap, but the left-top corner can be adjusted for each subplot
    pub fn with_settings(
        data: Vec<(Key, Data<Color>)>,
        settings: MultiBitmapWidgetSettings,
    ) -> Self {
        let MultiBitmapWidgetSettings {
            start_size,
            id: debug_name,
            boundary_between_data,
            colorbar,
            background,
            boundary_unselected,
            boundary_selected,
            boundary_factor_min,
        } = settings;
        Self {
            showmap: crate::multimap::ShowMultiMap::with_settings(
                data.into_iter()
                    .map(|(key, data)| crate::multimap::DataWithMetadata { key, data })
                    .collect(),
                crate::multimap::ShowMultiMapSettings {
                    boundary_between_data,
                    colorbar,
                    background,
                    boundary_unselected,
                    boundary_selected,
                    boundary_factor_min,
                },
            ),
            current_size: start_size.unwrap_or_default(),
            dynamic_resizing: start_size.is_none(),
            rendered_image: RenderedImage::from_color_image(
                debug_name.clone(),
                egui::ColorImage::new([3, 3], Color::GOLD),
            ),
            needs_rendering: true,
            debug_name,
            hide_key: None,
            copy_to_clipboard_delay: None,
        }
    }

    fn convert_window2multimap(
        &self,
        rect: egui::Rect,
        pos: Option<egui::Pos2>,
        size: [f32; 2],
    ) -> Option<crate::multimap::MultiMapPoint> {
        let (x, y) = Self::window2rect(rect, pos?)?;
        if x < 0. || y < 0. || x > 1. || y > 1. {
            None
        } else {
            let x = (size[0] * x) as usize;
            let y = (size[1] * y) as usize;
            if x >= size[0] as usize || y >= size[1] as usize {
                None
            } else {
                Some(crate::multimap::MultiMapPoint { x, y })
            }
        }
    }
    fn window2rect(rect: egui::Rect, egui::Pos2 { x, y }: egui::Pos2) -> Option<(f32, f32)> {
        let egui::Pos2 { x: ltx, y: lty } = rect.left_top();
        let egui::Pos2 { x: brx, y: bry } = rect.right_bottom();
        let x = (x - ltx) / (brx - ltx);
        let y = (y - lty) / (bry - lty);
        if x < 0. || y < 0. || x > 1. || y > 1. {
            None
        } else {
            Some((x, y))
        }
    }
    fn convert_window2bitmap(
        &self,
        rect: egui::Rect,
        pos: Option<egui::Pos2>,
        size: [f32; 2],
        state: &crate::multimap::MultimapState<Key>,
    ) -> MultiMapPosition<Key> {
        if let Some(multimap_point) = self.convert_window2multimap(rect, pos, size) {
            self.showmap.convert_multimap2bitmap(
                multimap_point,
                [size[0] as usize, size[1] as usize],
                state,
            )
        } else {
            MultiMapPosition::NotHovering
        }
    }
    /// Show widget
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut ShowState<Key>) {
        let shown_before = state.currently_showing();
        if let Some((before, size)) = self.copy_to_clipboard_delay {
            let now = std::time::Instant::now();
            if now - before > COPY_CLIPBOARD_DELAY {
                self.copy_to_clipboard_delay = None;
                self.copy_to_clipboard(size, state);
            }
        }
        let size = self.update_size(ui.available_size());
        self.render(state);
        let rendered = self.rendered_image.texture_id(ui.ctx());
        let image = egui::Widget::ui(
            egui::Image::new(rendered, size).sense(egui::Sense::click_and_drag()),
            ui,
        );

        let mouse = image.hover_pos();
        let rect = image.rect;
        state.mouse = self.convert_window2bitmap(rect, mouse, size, &state.multimap);
        let mouse_pos = state.mouse.get_pos().cloned();

        let image = image.context_menu(|ui| {
            ui.vertical(|ui| {
                if ui.button(&state.localization.text_home).clicked() {
                    self.showmap.home(state.get_inner_mut());
                    self.needs_rendering = true;
                    ui.close_menu();
                }
                if ui.button(&state.localization.text_unselect_all).clicked() {
                    if state.unselect_all() {
                        self.needs_rendering = true;
                    }
                    ui.close_menu();
                }

                if state.has_hidden() && ui.button(&state.localization.text_show_all).clicked() {
                    state.show_all();
                    self.needs_rendering = true;
                    ui.close_menu()
                }
                if let Some(key) = state.mouse.get_key() {
                    if state.can_hide() {
                        self.hide_key = Some(key.clone());
                    }
                }
                if let Some(key) = &self.hide_key {
                    if ui.button(&state.localization.text_hide).clicked() {
                        state.hide(key);
                        self.needs_rendering = true;
                        self.hide_key = None;
                        ui.close_menu()
                    }
                }
                if ui
                    .button(&state.localization.text_copy_to_clipboard_instantly)
                    .clicked()
                {
                    self.copy_to_clipboard(size, state);
                    ui.close_menu()
                }
                if ui
                    .button(&state.localization.text_copy_to_clipboard_delayed)
                    .clicked()
                {
                    self.copy_to_clipboard_delay = Some((std::time::Instant::now(), size));
                    ui.ctx().request_repaint_after(COPY_CLIPBOARD_DELAY);
                    ui.close_menu()
                }
            });
        });

        state.clicked = false;

        if image.double_clicked() {
            if let Some(pos) = &mouse_pos {
                self.showmap.center_to(pos, state.change_rect());
                self.needs_rendering = true;
            }
        } else if image.clicked() {
            if let Some(pos) = &mouse_pos {
                state.clicked = true;
                self.showmap.select(
                    pos,
                    ui.ctx().input(|x| x.modifiers.ctrl),
                    state.change_selected(),
                );
                self.needs_rendering = true;
            }
        }
        if image.drag_started() {
            if let Some(pos) = &mouse_pos {
                self.showmap.drag_start(pos);
                self.needs_rendering = true;
            }
        } else if image.drag_released() {
            if let Some(pos) = &mouse_pos {
                self.showmap.drag_release(Some(pos), state.change_rect());
            } else {
                self.showmap.drag_release(None, state.change_rect());
            }
            self.needs_rendering = true;
        } else if image.dragged() {
            if let Some(pos) = &mouse_pos {
                if self.showmap.drag_is_ongoing(pos) {
                    self.needs_rendering = true;
                }
            }
        }

        // keyboard movement and zoom and homeing
        if image.hovered() && ui.ctx().memory(|x| x.focus().is_none()) {
            if let Some((key, modifiers)) = ui.ctx().input(|x| {
                let keys = &x.keys_down;
                if keys.len() == 1 {
                    Some((*keys.iter().next().unwrap(), x.modifiers))
                } else {
                    None
                }
            }) {
                // keyboard navigation
                for (needed_key, direction) in [
                    (egui::Key::ArrowDown, KeyBoardDirection::Down),
                    (egui::Key::ArrowUp, KeyBoardDirection::Up),
                    (egui::Key::ArrowRight, KeyBoardDirection::Right),
                    (egui::Key::ArrowLeft, KeyBoardDirection::Left),
                ] {
                    if key == needed_key && modifiers.is_none() {
                        self.showmap
                            .translate_keyboard(direction, state.change_rect());
                        self.needs_rendering = true;
                        break;
                    }
                }
                // keyboard zoom
                for (needed_key, zoom_increment) in
                    [(egui::Key::PlusEquals, 1), (egui::Key::Minus, -1)]
                {
                    if key == needed_key && modifiers.is_none() {
                        self.showmap.zoom(zoom_increment, state.change_rect());
                        self.needs_rendering = true;
                        break;
                    }
                }
                if modifiers.is_none() && key == egui::Key::Home {
                    self.showmap.home(state.get_inner_mut());
                    self.needs_rendering = true;
                }
            };
        }
        // mouse scroll
        if image.hovered() {
            let (scroll_delta, modifiers) = ui.ctx().input(|x| (x.scroll_delta, x.modifiers));
            let scroll_delta = if modifiers.shift {
                scroll_delta.x * 5. //TODO: make this magnifier configurable
            } else {
                scroll_delta.y
            };
            let scroll_delta = (scroll_delta / 50.).round() as i32; // TODO: Does this 50 depend on my machine/mouse/...
            if scroll_delta != 0 {
                if let Some(before) = self
                    .convert_window2bitmap(rect, mouse, size, &state.multimap)
                    .get_pos()
                {
                    self.showmap.zoom(scroll_delta, state.change_rect());
                    self.needs_rendering = true;
                    if let Some(after) = self
                        .convert_window2bitmap(rect, mouse, size, &state.multimap)
                        .get_pos()
                    {
                        self.showmap.translate(
                            CoordinatePoint {
                                x: before.x - after.x,
                                y: before.y - after.y,
                            },
                            state.change_rect(),
                        )
                    }
                }
            }
        }
        // shown area changed
        if state.currently_showing() != shown_before {
            state.events.push(Event::ShowRectangle);
        }
    }

    fn update_size(&mut self, available_size: egui::Vec2) -> [f32; 2] {
        if self.dynamic_resizing {
            let new_size = [available_size.x, available_size.y];
            if self.current_size != new_size {
                self.current_size = new_size;
                self.needs_rendering = true;
            }
            new_size
        } else {
            self.current_size
        }
    }

    fn render(&mut self, state: &mut ShowState<Key>) {
        if self.needs_rendering {
            self.needs_rendering = false;
            let w = self.current_size[0] as usize;
            let h = self.current_size[1] as usize;
            let (image, problem) = match self.showmap.render(w, h, &mut state.multimap) {
                Ok(image) => (
                    egui::ColorImage {
                        size: [w, h],
                        pixels: image,
                    },
                    None,
                ),
                Err(err) => (egui::ColorImage::new([w, h], Color::GOLD), Some(err)),
            };
            state.render_problem = problem;
            self.rendered_image = RenderedImage::from_color_image(self.debug_name.clone(), image);
        }
    }

    fn copy_to_clipboard(&self, size: [f32; 2], state: &mut ShowState<Key>) {
        let width = size[0] as usize;
        let height = size[1] as usize;
        match self.showmap.render(width, height, &mut state.multimap) {
            Ok(data) => {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(_clip) = clipboard_win::Clipboard::new_attempts(10) {
                        if let Some(fmt) = clipboard_win::register_format("PNG") {
                            let image = image::ImageBuffer::from_fn(
                                size[0] as u32,
                                size[1] as u32,
                                |x, y| {
                                    let c = data[(size[0] as u32 * y + x) as usize];
                                    let (r, g, b, _a) = c.to_tuple();
                                    image::Rgb([r, g, b])
                                },
                            );

                            let mut writer = std::io::Cursor::new(Vec::new());
                            if let Err(e) =
                                image.write_to(&mut writer, image::ImageOutputFormat::Png)
                            {
                                panic!("Failed to convert to png: {e}")
                            };
                            let image = writer.into_inner();
                            if let Err(e) = clipboard_win::raw::set(fmt.into(), &image) {
                                panic!("Failed to copy to clipboard: {e}");
                            }
                        }
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    let bytes = data
                        .into_iter()
                        .flat_map(|x| x.to_array())
                        .collect::<Vec<_>>();
                    let mut clipboard = arboard::Clipboard::new().unwrap();
                    let r = clipboard.set_image(arboard::ImageData {
                        width,
                        height,
                        bytes: bytes.into(),
                    });
                    if let Err(e) = r {
                        panic!("Failed to copy to clipboard: {e}");
                    }
                }
            }
            Err(_) => todo!(),
        }
        /*
            fn render_to_buffer(&mut self, size: [f32; 2]) -> Option<Vec<u8>> {
            if let Ok(image) = self.showmap.render(size[0] as usize, size[1] as usize) {
                let image = image::ImageBuffer::from_fn(size[0] as u32, size[1] as u32, |x, y| {
                    let c = image[(size[0] as u32 * y + x) as usize];
                    let (r, g, b, _a) = c.to_tuple();
                    image::Rgb([r, g, b])
                });

                let mut writer = std::io::Cursor::new(Vec::new());
                if let Err(e) = image.write_to(&mut writer, image::ImageOutputFormat::Png) {
                    panic!("Failed to convert to png: {e}")
                };
                Some(writer.into_inner())
            } else {
                None
            }
        } */
    }
}
