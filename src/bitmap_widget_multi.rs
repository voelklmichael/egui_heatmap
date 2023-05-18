use std::fmt::Debug;

use crate::multimap::KeyBoardDirection;
pub use crate::multimap::{
    BitMapText, ColorWithThickness, CoordinatePoint, CoordinateRect, Data, FontOptions, Overlay,
    RenderProblem,
};
use egui::Color32 as Color;
use egui_extras::RetainedImage as RenderedImage;

/// Hover type
#[derive(Debug, Clone)]
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
    render_problem: Option<RenderProblem>,
    // size
    current_size: [f32; 2],
    dynamic_resizing: bool,
    // egui
    rendered_image: RenderedImage,
    debug_name: String,
    needs_rendering: bool,
    // interaction
    mouse: MultiMapPosition<Key>,
    clicked: bool, // Clicked plot can be fetched by mouse-value
    hide_key: Option<Key>,
    copy_to_clipboard_delay: Option<(std::time::Instant, [f32; 2])>,
    // events
    events: Vec<Event<Key>>,
}

/// Type for visibility events
pub enum Event<Key> {
    /// Show all plots
    ShowAll,
    /// Hide a single plot
    Hide(Key),
    /// Hide all plots except a single one
    HideAllExcept(Key),
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

impl<Key: Clone + PartialEq + Debug> MultiBitmapWidget<Key> {
    /// Get events
    pub fn fetch_events(&mut self) -> Vec<Event<Key>> {
        std::mem::take(&mut self.events)
    }
    /// Fetch array which is currently shown
    pub fn currently_showing(&self) -> CoordinateRect {
        self.showmap.currently_showing()
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
                    .map(|(key, data)| crate::multimap::DataWithMetadata {
                        key,
                        data,
                        visible: true,
                    })
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
            render_problem: None,
            mouse: MultiMapPosition::NotHovering,
            clicked: false,
            hide_key: None,
            copy_to_clipboard_delay: None,
            events: Default::default(),
        }
    }

    /// Check if widget is hovered
    pub fn hover(&self) -> MultiMapPosition<Key> {
        self.mouse.clone()
    }
    /// Check if there was a problem during the last rendering pass
    pub fn problem(&self) -> Option<RenderProblem> {
        self.render_problem.clone()
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
    ) -> MultiMapPosition<Key> {
        if let Some(multimap_point) = self.convert_window2multimap(rect, pos, size) {
            self.showmap
                .convert_multimap2bitmap(multimap_point, [size[0] as usize, size[1] as usize])
        } else {
            MultiMapPosition::NotHovering
        }
    }
    /// Show widget
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some((before, size)) = self.copy_to_clipboard_delay {
            let now = std::time::Instant::now();
            if now - before > COPY_CLIPBOARD_DELAY {
                self.copy_to_clipboard_delay = None;
                let image = self.render_to_buffer(size);
                if let Some(image) = image {
                    copy_png_to_clipboard(&image);
                }
            }
        }
        let size = self.update_size(ui.available_size());
        self.render();
        let image = ui.image(self.rendered_image.texture_id(ctx), size);

        let image = image.interact(egui::Sense::click_and_drag());

        let mouse = image.hover_pos();
        let rect = image.rect;
        self.mouse = self.convert_window2bitmap(rect, mouse, size);

        let image = image.context_menu(|ui| {
            ui.vertical(|ui| {
                if ui.button("Home").clicked() {
                    self.showmap.home();
                    self.needs_rendering = true;
                    ui.close_menu();
                }
                if ui.button("Unselect all").clicked() {
                    if self.showmap.unselect_all() {
                        self.needs_rendering = true;
                    }
                    ui.close_menu();
                }

                if self.showmap.has_hidden() {
                    if ui.button("Show all").clicked() {
                        self.showmap.show_all();
                        self.events.push(Event::ShowAll);
                        self.needs_rendering = true;
                        ui.close_menu()
                    }
                }
                if let Some(key) = self.mouse.get_key() {
                    if self.showmap.can_hide() {
                        self.hide_key = Some(key.clone());
                    }
                }
                if let Some(key) = &self.hide_key {
                    if ui.button("Hide").clicked() {
                        self.showmap.hide(key);
                        self.events.push(Event::Hide(key.clone()));
                        self.needs_rendering = true;
                        self.hide_key = None;
                        ui.close_menu()
                    }
                }
                if ui.button("Copy to Clipboard").clicked() {
                    let image = self.render_to_buffer(size);
                    if let Some(image) = image {
                        copy_png_to_clipboard(&image);
                    }
                    ui.close_menu()
                }
                if ui.button("Copy to Clipboard in 3 seconds").clicked() {
                    self.copy_to_clipboard_delay = Some((std::time::Instant::now(), size));
                    ctx.request_repaint_after(COPY_CLIPBOARD_DELAY);
                    ui.close_menu()
                }
            });
        });

        self.clicked = false;

        if image.double_clicked() {
            if let Some(pos) = self.mouse.get_pos() {
                self.showmap.center_to(pos);
                self.needs_rendering = true;
            }
        } else if image.clicked() {
            if let Some(pos) = self.mouse.get_pos() {
                self.clicked = true;
                self.showmap.select(pos, ctx.input(|x| x.modifiers.ctrl));
                self.needs_rendering = true;
            }
        }
        if image.drag_started() {
            if let Some(pos) = self.mouse.get_pos() {
                self.showmap.drag_start(pos);
                self.needs_rendering = true;
            }
        } else if image.drag_released() {
            if let Some(pos) = self.mouse.get_pos() {
                self.showmap.drag_release(Some(pos));
            } else {
                self.showmap.drag_release(None);
            }
            self.needs_rendering = true;
        } else if image.dragged() {
            if let Some(pos) = self.mouse.get_pos() {
                if self.showmap.drag_is_ongoing(pos) {
                    self.needs_rendering = true;
                }
            }
        }

        // keyboard movement and zoom and homeing
        if image.hovered() && ctx.memory(|x| x.focus().is_none()) {
            if let Some((key, modifiers)) = ctx.input(|x| {
                let keys = &x.keys_down;
                if keys.len() == 1 {
                    Some((keys.iter().next().unwrap().clone(), x.modifiers.clone()))
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
                        self.showmap.translate_keyboard(direction);
                        self.needs_rendering = true;
                        break;
                    }
                }
                // keyboard zoom
                for (needed_key, zoom_increment) in
                    [(egui::Key::PlusEquals, 1), (egui::Key::Minus, -1)]
                {
                    if key == needed_key && modifiers.is_none() {
                        self.showmap.zoom(zoom_increment);
                        self.needs_rendering = true;
                        break;
                    }
                }
                if modifiers.is_none() && key == egui::Key::Home {
                    self.showmap.home();
                    self.needs_rendering = true;
                }
            };
        }
        // mouse scroll
        if image.hovered() {
            let (scroll_delta, modifiers) = ctx.input(|x| (x.scroll_delta, x.modifiers));
            let scroll_delta = if modifiers.shift {
                scroll_delta.x * 5. //TODO: make this magnifier configurable
            } else {
                scroll_delta.y
            };
            let scroll_delta = (scroll_delta / 50.).round() as i32; // TODO: Does this 50 depend on my machine/mouse/...
            if scroll_delta != 0 {
                if let Some(before) = self.convert_window2bitmap(rect, mouse, size).get_pos() {
                    self.showmap.zoom(scroll_delta);
                    self.needs_rendering = true;
                    if let Some(after) = self.convert_window2bitmap(rect, mouse, size).get_pos() {
                        self.showmap.translate(CoordinatePoint {
                            x: before.x - after.x,
                            y: before.y - after.y,
                        })
                    }
                }
            }
        }
    }

    fn render_to_buffer(&mut self, size: [f32; 2]) -> Option<Vec<u8>> {
        let image = if let Ok(image) = self.showmap.render(size[0] as usize, size[1] as usize) {
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
        };
        image
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

    fn render(&mut self) {
        if self.needs_rendering {
            self.needs_rendering = false;
            let w = self.current_size[0] as usize;
            let h = self.current_size[1] as usize;
            let (image, problem) = match self.showmap.render(w, h) {
                Ok(image) => (
                    egui::ColorImage {
                        size: [w, h],
                        pixels: image,
                    },
                    None,
                ),
                Err(err) => (egui::ColorImage::new([w, h], Color::GOLD), Some(err)),
            };
            self.render_problem = problem;
            self.rendered_image = RenderedImage::from_color_image(self.debug_name.clone(), image);
        }
    }

    /// Get the currently selected points
    pub fn selected(&self) -> impl ExactSizeIterator<Item = &CoordinatePoint> {
        self.showmap.selected()
    }
}

fn copy_png_to_clipboard(image: &[u8]) {
    #[cfg(target_os = "windows")]
    if let Ok(_clip) = clipboard_win::Clipboard::new_attempts(10) {
        if let Some(fmt) = clipboard_win::register_format("PNG") {
            if let Err(e) = clipboard_win::raw::set(fmt.into(), image) {
                panic!("Failed to copy to clipboard: {e}");
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut clipboard = Clipboard::new().unwrap();
        c
    }
}
