// This modules contains some basic color types and constants
use egui::Color32 as Color;
pub use oklab::Oklab;

/// fetch one of the distinguishable colors
/// this loops around automatically
pub fn get_distinguishable_color(index: usize) -> Color {
    DISTINGUISHABLE_COLORS[index % DISTINGUISHABLE_COLORS.len()]
}
/// This is a list of distinguisable colors.
/// Source: https://en.m.wikipedia.org/wiki/Help:Distinguishable_colors
pub const DISTINGUISHABLE_COLORS: [Color; 26] = [
    //egui::Color32::from_rgb(255, 255, 255), // White
    egui::Color32::from_rgb(240, 163, 255), // Amethyst
    egui::Color32::from_rgb(0, 117, 220),   // Blue
    egui::Color32::from_rgb(153, 63, 0),    // Caramel
    egui::Color32::from_rgb(76, 0, 92),     // Damson
    egui::Color32::from_rgb(25, 25, 25),    // Ebony
    egui::Color32::from_rgb(0, 92, 49),     // Forest
    egui::Color32::from_rgb(43, 206, 72),   // Green
    egui::Color32::from_rgb(255, 204, 153), // Honeydew
    egui::Color32::from_rgb(128, 128, 128), // Iron
    egui::Color32::from_rgb(148, 255, 181), // Jade
    egui::Color32::from_rgb(143, 124, 0),   // Khaki
    egui::Color32::from_rgb(157, 204, 0),   // Lime
    egui::Color32::from_rgb(194, 0, 136),   // Mallow
    egui::Color32::from_rgb(0, 51, 128),    // Navy
    egui::Color32::from_rgb(255, 164, 5),   // Orpiment
    egui::Color32::from_rgb(255, 168, 187), // Pink
    egui::Color32::from_rgb(66, 102, 0),    // Quagmire
    egui::Color32::from_rgb(255, 0, 16),    // Red
    egui::Color32::from_rgb(94, 241, 242),  // Sky
    egui::Color32::from_rgb(0, 153, 143),   // Turquoise
    egui::Color32::from_rgb(224, 255, 102), // Uranium
    egui::Color32::from_rgb(116, 10, 255),  // Violet
    egui::Color32::from_rgb(153, 0, 0),     // Wine
    egui::Color32::from_rgb(255, 255, 128), // Xanthin
    egui::Color32::from_rgb(255, 225, 0),   // Yellow
    egui::Color32::from_rgb(255, 80, 5),    // Zinnia
];

/// Options for gradient gradient
pub enum ColorGradientOptions {
    /// Linear gradient from start to end
    StartEnd {
        /// Start color
        start: Color,
        /// end color
        end: Color,
        /// steps
        steps: usize,
    },
    /// Linear gradient from start to center, combined with linear gradient from center to end
    StartCenterEnd {
        /// Start color
        start: Color,
        /// Center color
        center: Color,
        /// End color
        end: Color,
        /// steps
        steps: usize,
    },
}
fn convert_to_oklab(egui: &Color) -> Oklab {
    let rgba = egui::Rgba::from(*egui);
    let [r, g, b, _a] = rgba.to_array();
    oklab::linear_srgb_to_oklab(oklab::RGB { r, g, b })
}
/// Convert an Oklab color to an egui-Color
pub fn convert_from_oklab(oklab: Oklab) -> Color {
    let rgb = oklab::oklab_to_linear_srgb(oklab);
    Color::from_rgb_additive(
        (rgb.r * 255.) as u8,
        (rgb.g * 255.) as u8,
        (rgb.b * 255.) as u8,
    )
}
fn interpolate(start: &Oklab, end: &Oklab, counts_minus_one: f32, i: f32) -> Color {
    let l = interpolate_single_channel(start.l, end.l, counts_minus_one, i);
    let a = interpolate_single_channel(start.a, end.a, counts_minus_one, i);
    let b = interpolate_single_channel(start.b, end.b, counts_minus_one, i);
    let oklab = Oklab { l, a, b };
    convert_from_oklab(oklab)
}
fn interpolate_single_channel(start: f32, end: f32, counts_minus_one: f32, i: f32) -> f32 {
    start + (end - start) * i / counts_minus_one
}
fn gradient(start: &Color, end: &Color, steps: usize) -> Vec<Color> {
    let start = convert_to_oklab(start);
    let end = convert_to_oklab(end);
    match steps {
        0 => Vec::new(),
        1 => vec![interpolate(&start, &end, 2., 1.)],
        n => {
            let counts_minus_one = (n - 1) as f32;
            (0..n)
                .map(|i| interpolate(&start, &end, counts_minus_one, i as f32))
                .collect()
        }
    }
}

/// Color Gradient
pub struct Gradient<C>(pub(crate) Vec<C>);
impl<C: Clone> Gradient<C> {
    pub(crate) fn element_at(&self, row: usize, height: usize) -> C {
        self.0[row * self.0.len() / height].clone()
    }
    /// Compute the color at a given ratio v in [0.0, 1.0]
    pub fn lookup_color(&self, v: f32) -> C {
        let Gradient(gradient) = self;
        let index = v * (gradient.len() as f32);
        let index = if index < 0. {
            0
        } else if index as usize >= gradient.len() {
            gradient.len() - 1
        } else {
            index as usize
        };
        gradient[index].clone()
    }

    pub(crate) fn fetch_value(&self, lower: f32, upper: f32, relative_distance: f32) -> f32 {
        let n = self.0.len();
        if n == 0 {
            f32::NAN
        } else if n == 1 {
            (lower + upper) / 2.
        } else {
            let relative_distance = if relative_distance < 0. {
                0.
            } else if relative_distance > 1. {
                1.
            } else {
                relative_distance
            };
            let delta = (upper - lower) / ((n - 1) as f32);
            let f = (relative_distance * n as f32).floor() * delta + lower;
            if f > upper {
                upper
            } else {
                f
            }
        }
    }
}
impl Gradient<Color> {
    /// This computes a color gradient
    pub fn with_options(options: &ColorGradientOptions) -> Self {
        Self(match options {
            ColorGradientOptions::StartEnd { start, end, steps } => gradient(start, end, *steps),
            ColorGradientOptions::StartCenterEnd {
                start,
                center,
                end,
                steps,
            } => match *steps {
                0 => vec![],
                1 => vec![*center],
                2 => vec![*start, *end],
                3 => vec![*start, *center, *end],
                n if n % 2 == 0 => {
                    let steps = n;
                    let mut start_center = gradient(start, center, steps);
                    let mut center_end = gradient(center, end, steps);
                    for i in 0..steps / 2 {
                        start_center.remove(i + 1);
                    }
                    for i in 0..steps / 2 {
                        center_end.remove(i);
                    }
                    start_center.extend(center_end);
                    start_center
                }
                n => {
                    let steps = (n + 1) / 2;
                    let mut start_center = gradient(start, center, steps);
                    let center_end = gradient(center, end, steps);
                    start_center.pop(); // remove center, which is at beginning of center_end and of end of start_center
                    start_center.extend(center_end);
                    start_center
                }
            },
        })
    }
}
