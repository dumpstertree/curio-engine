use core::fmt;
use fmt::Display;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Mul;

/// Represents an r,g,b,a color
///
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
// const
impl Color {
    /// Creates an instance of Color using 0.0 - 1.0 values to represeent r,g,b,a
    pub const fn new_01(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: r, g: g, b: b, a: a }
    }
    /// creates an instance of Color using 0 - 255 values to represeent r,g,b,a
    pub const fn new_0255(r: i32, g: i32, b: i32, a: i32) -> Color {
        Color {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }
    /// creates an instance of Color using a hexcode to represeent r,g,b,a. '#' is optional. Accepts color lengths of 6 or 8. Returns Color::clear() on failure
    pub const fn new_hex(hex: &str) -> Color {
        let bytes = hex.as_bytes();
        // replace `bytes.get(0)` with an explicit length check + indexing
        let start = if bytes.len() >= 1 && bytes[0] == b'#' { 1 } else { 0 };
        let len = bytes.len() - start;

        match len {
            6 => Color::new_0255(Color::hex_pair_to_u8(bytes, start) as i32, Color::hex_pair_to_u8(bytes, start + 2) as i32, Color::hex_pair_to_u8(bytes, start + 4) as i32, 255),
            8 => Color::new_0255(
                Color::hex_pair_to_u8(bytes, start) as i32,
                Color::hex_pair_to_u8(bytes, start + 2) as i32,
                Color::hex_pair_to_u8(bytes, start + 4) as i32,
                Color::hex_pair_to_u8(bytes, start + 6) as i32,
            ),
            _ => Color::clear(),
        }
    }
    /// creates an instance of Color with the values (0.0, 0.0, 0.0, 0.0)
    pub const fn clear() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    /// creates an instance of Color with the values (0.0, 0.0, 0.0, 1.0)
    pub const fn black() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
    /// creates an instance of Color with the values (1.0, 1.0, 1.0, 1.0)
    pub const fn white() -> Color {
        Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
    /// creates an instance of Color with the values (1.0, 0.0, 0.0, 1.0)
    pub const fn red() -> Color {
        Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    }
    /// creates an instance of Color with the values (0.0, 1.0, 0.0, 1.0)
    pub const fn green() -> Color {
        Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    }
    /// creates an instance of Color with the values (0.0, 0.0, 1.0, 1.0)
    pub const fn blue() -> Color {
        Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
    }
}

// static
impl Color {
    const fn hex_pair_to_u8(bytes: &[u8], idx: usize) -> u8 {
        let hi = Color::hex_char_to_val(bytes[idx]);
        let lo = Color::hex_char_to_val(bytes[idx + 1]);
        (hi << 4) | lo
    }
    const fn u8_to_hex_pair(val: u8) -> (u8, u8) {
        const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
        let hi = HEX_CHARS[(val >> 4) as usize];
        let lo = HEX_CHARS[(val & 0xF) as usize];
        (hi, lo)
    }
    const fn hex_char_to_val(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => 0,
        }
    }
}

// whole num mult
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, x: f32) -> Color {
        Color::new_01(self.r * x, self.g * x, self.b * x, self.a * x)
    }
}
// whole num mult
impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, x: Color) -> Color {
        Color::new_01(self.r * x.r, self.g * x.g, self.b * x.b, self.a * x.a)
    }
} // instance
impl Color {
    pub fn as_r_01(&self) -> f32 {
        self.r
    }
    pub fn as_g_01(&self) -> f32 {
        self.g
    }
    pub fn as_b_01(&self) -> f32 {
        self.b
    }
    pub fn as_a_01(&self) -> f32 {
        self.a
    }
    pub fn as_r_0255(&self) -> i32 {
        (self.r * 255.0).round() as i32
    }
    pub fn as_g_0255(&self) -> i32 {
        (self.g * 255.0).round() as i32
    }
    pub fn as_b_0255(&self) -> i32 {
        (self.b * 255.0).round() as i32
    }
    pub fn as_a_0255(&self) -> i32 {
        (self.a * 255.0).round() as i32
    }
    pub fn as_hex(&self) -> String {
        let mut buf = [0u8; 9];
        buf[0] = b'#';

        let (hi, lo) = Color::u8_to_hex_pair((self.r * 255.0) as u8);
        buf[1] = hi;
        buf[2] = lo;

        let (hi, lo) = Color::u8_to_hex_pair((self.g * 255.0) as u8);
        buf[3] = hi;
        buf[4] = lo;

        let (hi, lo) = Color::u8_to_hex_pair((self.b * 255.0) as u8);
        buf[5] = hi;
        buf[6] = lo;

        let (hi, lo) = Color::u8_to_hex_pair((self.a * 255.0) as u8);
        buf[7] = hi;
        buf[8] = lo;

        str::from_utf8(&buf).unwrap().to_string()
    }

    pub fn set_r_01(&mut self, r: f32) {
        self.r = r.clamp(0.0, 1.0);
    }
    pub fn set_g_01(&mut self, g: f32) {
        self.g = g.clamp(0.0, 1.0);
    }
    pub fn set_b_01(&mut self, b: f32) {
        self.b = b.clamp(0.0, 1.0);
    }
    pub fn set_a_01(&mut self, a: f32) {
        self.a = a.clamp(0.0, 1.0);
    }
    pub fn set_r_0255(&mut self, r: i32) {
        self.r = (r as f32) / 255.0;
    }
    pub fn set_g_0255(&mut self, g: i32) {
        self.g = (g as f32) / 255.0;
    }
    pub fn set_b_0255(&mut self, b: i32) {
        self.b = (b as f32) / 255.0;
    }
    pub fn set_a_0255(&mut self, a: i32) {
        self.a = (a as f32) / 255.0;
    }
    pub fn set_hex(&mut self, hex: &str) {
        let bytes = hex.as_bytes();
        // replace `bytes.get(0)` with an explicit length check + indexing
        let start = if bytes.len() >= 1 && bytes[0] == b'#' { 1 } else { 0 };
        let len = bytes.len() - start;

        match len {
            6 => {
                self.r = Color::hex_pair_to_u8(bytes, start) as f32 / 255.0;
                self.g = Color::hex_pair_to_u8(bytes, start + 2) as f32 / 255.0;
                self.b = Color::hex_pair_to_u8(bytes, start + 4) as f32 / 255.0;
                self.a = 1.0;
            }
            8 => {
                self.r = Color::hex_pair_to_u8(bytes, start) as f32 / 255.0;
                self.g = Color::hex_pair_to_u8(bytes, start + 2) as f32 / 255.0;
                self.b = Color::hex_pair_to_u8(bytes, start + 4) as f32 / 255.0;
                self.a = Color::hex_pair_to_u8(bytes, start + 6) as f32 / 255.0;
            }
            _ => {
                self.r = 0.0;
                self.g = 0.0;
                self.b = 0.0;
                self.a = 0.0;
            }
        }
    }
}
// display
impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "r:{}, g: {}, b:{}, a:{}", self.r, self.g, self.b, self.a)
    }
}
