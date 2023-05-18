/// Font to use
#[derive(Debug, Clone, Default)]
pub enum Font {
    /// Use the highest-priority monospace font from egui
    #[default]
    EguiMonospace,
    /// Use a port of Font8x8, Font8x8-rs
    Font8x8,
}

/// Options for rendering a string
#[derive(Debug, Clone, Default)]
pub struct FontOptions {
    /// Font to use
    pub font: Font,
    /// Is the background transparent? Otherwise, background is black.
    pub background_is_transparent: bool,
    /// Height of font. Doubling this doubles the size of the rendered string (up to rounding/quantization)
    pub font_height: f32,
}
impl FontOptions {
    /// Render some text to a bitmap.
    /// Returns None in case of a problem
    pub fn render(&self, text: &str) -> Option<BitMapText> {
        BitMapText::new(text, self)
    }
}

/// A rendered gray-scale bitmap, representing a string rendered using some font
#[derive(PartialEq)]
pub struct BitMapText {
    /// data of the bitmap
    pub data: Vec<u8>,
    /// width of the bitmap
    pub width: i32,
    /// height of the bitmap
    pub height: i32,
}

impl BitMapText {
    /// Render some text
    /// The FontOptions::background_is_transparent is actually not used here
    pub fn new(
        text: &str,
        FontOptions {
            font_height,
            font,
            background_is_transparent: _,
        }: &FontOptions,
    ) -> Option<BitMapText> {
        let fonts = egui::FontDefinitions::default();
        match &font {
            Font::EguiMonospace => {
                let font = fonts
                    .families
                    .get(&egui::FontFamily::Monospace)
                    .and_then(|x| x.first())
                    .and_then(|label| fonts.font_data.get(label))
                    .and_then(|font| rusttype::Font::try_from_bytes(&font.font))
                    .expect("Failed to retrieve egui default font");
                /*fonts
                .families
                .get(&egui::FontFamily::Proportional)
                .map(|x| x.first())
                .flatten()
                .or(fonts
                    .families
                    .get(&egui::FontFamily::Monospace)
                    .map(|x| x.first())
                    .flatten())
                .map(|label| fonts.font_data.get(label))
                .flatten()
                .map(|font| rusttype::Font::try_from_bytes(&font.font as &[u8]))
                .flatten()*/

                // taken from RustType example
                // source: https://github.com/redox-os/rusttype/blob/master/dev/examples/ascii.rs

                // Desired font pixel height
                let height: f32 = *font_height; // to get 80 chars across (fits most terminals); adjust as desired
                let pixel_height = height.ceil() as usize;

                // 2x scale in x direction to counter the aspect ratio of monospace characters.
                let scale = rusttype::Scale {
                    x: height * 2.0,
                    y: height,
                };

                // The origin of a line of text is at the baseline (roughly where
                // non-descending letters sit). We don't want to clip the text, so we shift
                // it down with an offset when laying it out. v_metrics.ascent is the
                // distance between the baseline and the highest edge of any glyph in
                // the font. That's enough to guarantee that there's no clipping.
                let v_metrics = font.v_metrics(scale);
                let offset = rusttype::point(0.0, v_metrics.ascent);

                // Glyphs to draw for "RustType". Feel free to try other strings.
                let glyphs: Vec<_> = font.layout(text, scale, offset).collect();

                // Find the most visually pleasing width to display
                let width = glyphs
                    .iter()
                    .rev()
                    .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
                    .next()
                    .unwrap_or(0.0)
                    .ceil() as usize;
                let mut data = vec![0; width * pixel_height];
                for g in glyphs {
                    if let Some(bb) = g.pixel_bounding_box() {
                        g.draw(|x, y, v| {
                            let v = (v * 255.).round().clamp(0., 255.);
                            let v = v as u8;
                            let x = x as i32 + bb.min.x;
                            let y = y as i32 + bb.min.y;
                            // There's still a possibility that the glyph clips the boundaries of the bitmap
                            if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                                let x = x as usize;
                                let y = y as usize;
                                data[x + y * width] = v;
                            }
                        })
                    }
                }

                Some(Self {
                    data,
                    width: width as i32,
                    height: height as i32,
                })
            }
            Font::Font8x8 => {
                let mut chars = Vec::new();
                for c in text.chars() {
                    let bitmap = if c.is_ascii() {
                        let c = c as usize;
                        font8x8::unicode::BASIC_UNICODE[c].1
                    } else {
                        font8x8::unicode::BOX_UNICODE[108].1
                    };
                    let mut columns = Vec::new();
                    for column in 0..8 {
                        let mut c: [bool; 8] = Default::default();
                        for row in 0..8 {
                            c[row] = (bitmap[row] & 1 << column) != 0;
                        }
                        columns.push(c);
                    }
                    while columns.last().map(|x| x.iter().all(|x| !*x)) == Some(true) {
                        columns.pop();
                    }
                    while columns.first().map(|x| x.iter().all(|x| !*x)) == Some(true) {
                        columns.remove(0);
                    }
                    if !columns.is_empty() {
                        chars.push(columns);
                    }
                }
                let mut columns = Vec::new();
                for c in chars {
                    columns.extend(c);
                    columns.push([false; 8]);
                }
                columns.pop(); // remove last empty column
                if columns.is_empty() {
                    None
                } else {
                    let scaling = {
                        let scaling = font_height.round();
                        let scaling = if scaling.is_finite() && scaling > 1. {
                            scaling
                        } else {
                            1.
                        };
                        scaling as usize
                    };
                    let width = columns.len() * scaling;
                    let height = 8 * scaling;
                    let mut data = Vec::new();
                    for y in 0..(8 * scaling) {
                        for x in 0..width {
                            let x = x / scaling;
                            let y = y / scaling;
                            let c = columns[x][y];
                            let c = if c { 255 } else { 0 };
                            data.push(c);
                        }
                    }
                    Some(Self {
                        data,
                        width: width as i32,
                        height: height as i32,
                    })
                }
            }
        }
    }
    pub(crate) fn fetch(&self, pixel_distance_l: i32, pixel_distance_t: i32) -> Option<u8> {
        if pixel_distance_l < 0
            || pixel_distance_t < 0
            || pixel_distance_l >= self.width
            || pixel_distance_t >= self.height
        {
            None
        } else {
            Some(self.data[(pixel_distance_t * self.width + pixel_distance_l) as usize])
        }
    }
}
