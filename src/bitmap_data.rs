#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// This represents a bitmap point, in data coordinates
pub struct BitMapPoint {
    /// Row of point
    pub x: i32,
    /// Column of point
    pub y: i32,
}
#[derive(Clone, Copy, Debug)]
/// This represents a vector, so an offset between to data coordinates
pub struct BitMapVec {
    /// Row offset
    pub x: i32,
    /// Column offset
    pub y: i32,
}
impl std::ops::Sub<BitMapPoint> for BitMapPoint {
    type Output = BitMapVec;

    fn sub(self, rhs: BitMapPoint) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::AddAssign<BitMapVec> for BitMapPoint {
    fn add_assign(&mut self, rhs: BitMapVec) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::Add<BitMapVec> for BitMapPoint {
    type Output = BitMapPoint;
    fn add(self, rhs: BitMapVec) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// This represents numeric data
pub struct HeatmapData {
    /// Width of the data set
    pub width: i32,
    /// Height of the data set
    pub height: i32,
    /// Data points, row by row.
    /// Use nan (or any non-finite value) for positions without data
    pub pixels: Vec<f32>,
}
impl HeatmapData {
    /// Get data of a point, if data is available
    pub fn get_data_at_point(&self, BitMapPoint { x, y }: BitMapPoint) -> Option<f32> {
        if x < 0 || y < 0 {
            return None;
        }
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(x + y * self.width) as usize])
    }
    /// Convert this to a bitmap, using a range
    pub fn to_bitmap(
        &self,
        limits: (f32, f32),
        options: crate::colors::ColorGradientOptions,
        background: egui::Color32,
    ) -> BitmapData {
        let gradient = crate::colors::Gradient::<egui::Color32>::with_options(&options);
        let HeatmapData {
            width,
            height,
            pixels,
        } = self;
        let delta = limits.1 - limits.0;
        let pixels = pixels
            .iter()
            .map(|&x| {
                if x.is_finite() {
                    let x = if x < limits.0 {
                        limits.0
                    } else if x > limits.1 {
                        limits.1
                    } else {
                        x
                    };
                    let x = (x - limits.0) / delta;
                    gradient.lookup_color(x)
                } else {
                    background.clone()
                }
            })
            .collect();
        BitmapData {
            width: *width,
            height: *height,
            pixels,
        }
    }
    /// Some demo data set
    pub fn example_circle(width: usize, height: usize) -> Self {
        let mut data = Vec::with_capacity(width * height);
        let center = [width / 2, height / 2];
        let max_distance = std::cmp::max(width / 2, height / 2) as f32;
        for h in 0..height {
            for w in 0..width {
                let distance = [center[0] as f32 - w as f32, center[1] as f32 - h as f32];
                let distance = distance[0] * distance[0] + distance[1] * distance[1];
                let distance = distance.sqrt() / max_distance;
                let mut distance = 1. - distance;
                if distance < 0. {
                    distance = 0.;
                } else if distance > 1. {
                    distance = 1.;
                }
                data.push(distance);
            }
        }
        Self {
            width: width as i32,
            height: height as i32,
            pixels: data,
        }
    }
}

/// This represents the data which shall be shown in the widget
pub struct BitmapData {
    /// Width of the data set
    pub width: i32,
    /// Height of the data set
    pub height: i32,
    /// Colors of the data points, row by row.
    pub pixels: Vec<egui::Color32>,
}
impl BitmapData {
    /// Get the color of a point, if data is available
    pub fn get_color_at_point(&self, BitMapPoint { x, y }: BitMapPoint) -> Option<egui::Color32> {
        if x < 0 || y < 0 {
            return None;
        }
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(x + y * self.width) as usize].clone())
    }
    /// Set the color of a point. Returns previous color, if any. Size of bitmap rectangular is not increased.
    pub fn set_color_at_point(
        &mut self,
        BitMapPoint { x, y }: BitMapPoint,
        c: egui::Color32,
    ) -> Option<egui::Color32> {
        if x < 0 || y < 0 {
            return None;
        }
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = (x + y * self.width) as usize;
        let old = self.pixels[i];
        self.pixels[i] = c;
        Some(old)
    }
}
