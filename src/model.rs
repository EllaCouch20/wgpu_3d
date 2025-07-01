use wgpu::{BindGroup, BufferAddress, VertexBufferLayout, VertexAttribute, VertexStepMode, VertexFormat, IndexFormat, RenderPass, Buffer, BufferUsages};
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::CameraContext;
use crate::LightContext;
use crate::texture::Texture;
use std::mem;

use crate::CanvasContext;

#[derive(Clone, Copy)]
pub struct Area3D(pub f32, pub f32, pub f32);

impl Area3D {
    pub fn position(&self) -> [f32; 3] {[self.0, self.1, self.2]}
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: BindGroup,
}

impl Material {
    pub async fn new(ctx: &mut CanvasContext, file_name: String, m: tobj::Material) -> Self {
        let diffuse_texture = ctx.load_texture(&m.diffuse_texture).await.unwrap();
        let bind_group = ctx.create_bind_group(&diffuse_texture);

        Material {
            name: file_name,
            diffuse_texture,
            bind_group,
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_elements: u32,
    pub material: usize,
}

impl Mesh {
    pub fn new(ctx: &mut CanvasContext, file_name: String, m: tobj::Model) -> Self {
        let vertices = (0..m.mesh.positions.len() / 3).map(|i| ModelVertex {
            position: [
                m.mesh.positions[i * 3],
                m.mesh.positions[i * 3 + 1],
                m.mesh.positions[i * 3 + 2],
            ],
            tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
            normal: [
                m.mesh.normals[i * 3],
                m.mesh.normals[i * 3 + 1],
                m.mesh.normals[i * 3 + 2],
            ],
        }).collect::<Vec<_>>();

        let vertex_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", file_name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", file_name)),
            contents: bytemuck::cast_slice(&m.mesh.indices),
            usage: BufferUsages::INDEX,
        });

        Mesh {
            name: file_name,
            vertex_buffer,
            index_buffer,
            num_elements: m.mesh.indices.len() as u32, 
            material: m.mesh.material_id.unwrap_or(0)
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub area: Area3D
}

impl Model {
    pub fn new(ctx: &mut CanvasContext, area: Area3D, models: Vec<tobj::Model>, materials: Vec<Material>, file_name: String) -> Self {
        let meshes = models.into_iter().map(|m| {
            Mesh::new(ctx, file_name.clone(), m)
        }).collect::<Vec<_>>();

        Self { meshes, materials, area }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a CameraContext, light: &'a LightContext) {
        render_pass.draw_model(&self, &camera.bind_group, &light.bind_group);
    }

    pub fn light<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &'a CameraContext, light: &'a LightContext) {
        render_pass.draw_light_model(&self, &camera.bind_group, &light.bind_group);
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material, camera: &'a BindGroup, light: &'a BindGroup);
    fn draw_model(&mut self, model: &'a Model, camera: &'a BindGroup, light: &'a BindGroup);
}

impl<'a, 'b> DrawModel<'b> for RenderPass<'a> where 'b: 'a {
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, camera: &'b BindGroup, light: &'b BindGroup,) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera, &[]);
        self.set_bind_group(2, light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_model(&mut self, model: &'b Model, camera: &'b BindGroup, light: &'a BindGroup) {
        model.meshes.iter().for_each(|mesh| self.draw_mesh(mesh, &model.materials[mesh.material], camera, light));
    }
}

pub trait DrawLight<'a> {
    fn draw_light_mesh(&mut self, mesh: &'a Mesh, camera: &'a BindGroup, light: &'a BindGroup);
    fn draw_light_model(&mut self, model: &'a Model, camera: &'a BindGroup, light: &'a BindGroup);
}

impl<'a, 'b> DrawLight<'b> for RenderPass<'a> where 'b: 'a {
    fn draw_light_mesh(&mut self, mesh: &'b Mesh, camera: &'b BindGroup, light: &'b BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        self.set_bind_group(0, camera, &[]);
        self.set_bind_group(1, light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    fn draw_light_model(&mut self, model: &'b Model, camera: &'b BindGroup, light: &'b BindGroup) {
        model.meshes.iter().for_each(|mesh| self.draw_light_mesh(mesh, camera, light));
    }
}
