pub trait GammyMultiplyable {
    fn gamma_multiply(self, factor: f32) -> Self;
}

impl GammyMultiplyable for char {
    fn gamma_multiply(self, _: f32) -> Self {
        self
    }
}

impl GammyMultiplyable for egui::Color32 {
    fn gamma_multiply(self, factor: f32) -> Self {
        self.gamma_multiply(factor)
    }
}

pub trait BitMapDrawable {
    fn gray(gray: u8) -> Self;
    fn saturating_add(&self, gray: u8) -> Self;
    fn remove_alpha(self) -> Self;
}

impl BitMapDrawable for char {
    fn gray(_: u8) -> Self {
        'g'
    }

    fn saturating_add(&self, _u: u8) -> Self {
        self.clone()
    }

    fn remove_alpha(self) -> Self {
        self
    }
}
impl BitMapDrawable for egui::Color32 {
    fn gray(gray: u8) -> Self {
        Self::from_additive_luminance(gray)
    }

    fn saturating_add(&self, gray: u8) -> Self {
        let c = self;
        Self::from_rgb(
            c.r().saturating_add(gray),
            c.g().saturating_add(gray),
            c.b().saturating_add(gray),
        )
    }
    fn remove_alpha(self) -> Self {
        let (r, g, b, _a) = self.to_tuple();
        Self::from_rgba_unmultiplied(r, g, b, 255)
    }
}
