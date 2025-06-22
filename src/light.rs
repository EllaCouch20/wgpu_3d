use crate::color::Color;
use crate::model::Area3D;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32,
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding2: u32,
}

impl LightUniform {
    pub fn new(position: Area3D, color: Color) -> Self {
        LightUniform {
            position: position.position(),
            _padding: 0,
            color: color.color(),
            _padding2: 0
        }
    }
}
