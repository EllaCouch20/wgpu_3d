use wgpu::{Surface, Device, Queue, SurfaceConfiguration, BindGroup, BindGroupLayout, BindGroupDescriptor, BindGroupEntry, BindingResource};
use std::io::{BufReader, Cursor};


use cfg_if::cfg_if;

use crate::texture;

use crate::texture::Texture;

use crate::model::Model;
use crate::model::Material;
use crate::model::Area3D;

pub struct Context {
    pub device: Device,
    pub queue: Queue,
    pub layout: BindGroupLayout,
    pub models: Vec<Model>,
    pub config: SurfaceConfiguration,
}

impl Context {
    pub fn new(device: Device, queue: Queue, layout: BindGroupLayout, config: SurfaceConfiguration) -> Self {
        Context { device, queue, layout, config, models: vec![] }
    }
    #[cfg(target_arch = "wasm32")]
    fn format_url(file_name: &str) -> reqwest::Url {
        let window = web_sys::window().unwrap();
        let location = window.location();
        let base = reqwest::Url::parse(&format!(
            "{}/{}/",
            location.origin().unwrap(),
            option_env!("RES_PATH").unwrap_or("assets"),
        ))
        .unwrap();
        base.join(file_name).unwrap()
    }

    pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                log::warn!("Load model on web");

                let url = format_url(file_name);
                let txt = reqwest::get(url)
                    .await?
                    .text()
                    .await?;

                log::warn!("{}", txt);

            } else {
                let path = std::path::Path::new("assets")
                    .join(file_name);
                let txt = std::fs::read_to_string(path)?;
            }
        }

        Ok(txt)
    }

    pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let url = format_url(file_name);
                let data = reqwest::get(url)
                    .await?
                    .bytes()
                    .await?
                    .to_vec();
            } else {
                let path = std::path::Path::new("assets")
                    .join(file_name);
                let data = std::fs::read(path)?;
            }
        }

        Ok(data)
    }

    pub async fn load_texture(&mut self, file_name: &str) -> anyhow::Result<texture::Texture> {
        let data = Self::load_binary(file_name).await?;
        texture::Texture::from_bytes(&self.device, &self.queue, &data, file_name)
    }

    pub async fn load_model(&mut self, file_name: &str, area: Area3D) -> Result<(), anyhow::Error> {
        let obj_text = Self::load_string(file_name).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = Self::load_string(&p).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        )
        .await?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            let material = Material::new(self, file_name.to_string(), m).await;
            materials.push(material);
        }

        let model = Model::new(self, area, models, materials, file_name.to_string());
        self.models.push(model);

        Ok(())
    }

    pub fn create_bind_group(&mut self, diffuse_texture: &Texture) -> BindGroup {
        self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        })
    }
}
