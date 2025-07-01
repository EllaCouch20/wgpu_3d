use wgpu::{RenderPipeline, BindGroup, Buffer};
use crate::color::Color;
use crate::model::Area3D;

pub struct Context {
    pub uniform: LightUniform,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub render_pipeline: RenderPipeline,
}

impl Context {
    pub fn new(lu: LightUniform, lb: Buffer, lbg: BindGroup, lrp: RenderPipeline) -> Self {
        Context {
            uniform: lu,
            buffer: lb,
            bind_group: lbg,
            render_pipeline: lrp,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
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
