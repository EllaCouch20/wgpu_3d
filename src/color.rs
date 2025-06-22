
#[derive(Clone, Copy, Debug)]
pub struct Color(pub f32, pub f32, pub f32);

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color(r, g, b)
    }

    // pub fn from_hex(color: &'static str, alpha: u8) -> Self {
    //     let ce = "Color was not a Hex Value";
    //     let c = hex::decode(color.strip_prefix('#').unwrap_or(color)).expect(ce);
    //     Color(c[0], c[1], c[2], alpha)
    // }

    // pub(crate) fn color(&self) -> [f32; 4] {
    //     let c = |f: u8| (((f as f32 / u8::MAX as f32) + 0.055) / 1.055).powf(2.4);
    //     [c(self.0), c(self.1), c(self.2), c(self.3)]
    // }

    pub(crate) fn color(&self) -> [f32; 3] {
        [self.0, self.1, self.2]
    }
}
