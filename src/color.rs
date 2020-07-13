use std::ops::{Mul, Add};

#[derive(Clone, Debug)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    pub fn clamp(&mut self) {
        self.red = if self.red > 1.0 { 1.0 } else { if self.red < 0.0 { 0.0 } else { self.red } };
        self.green = if self.green > 1.0 { 1.0 } else { if self.green < 0.0 { 0.0 } else { self.green } };
        self.blue = if self.blue > 1.0 { 1.0 } else { if self.blue < 0.0 { 0.0 } else { self.blue } };
    }

    pub fn to_rgba(&self) -> image::Rgba<u8> {
        assert!(self.red <= 1.0 && self.red >= 0.0);
        assert!(self.green <= 1.0 && self.red >= 0.0);
        assert!(self.blue <= 1.0 && self.red >= 0.0);
        image::Rgba([(self.red * 255.0) as u8, (self.green * 255.0) as u8, (self.blue * 255.0) as u8, 1])
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Color {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl Add<f32> for Color {
    type Output = Self;

    fn add(self, rhs: f32) -> Self {
        Color {
            red: self.red + rhs,
            green: self.green + rhs,
            blue: self.blue + rhs,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Color {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Color {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}
