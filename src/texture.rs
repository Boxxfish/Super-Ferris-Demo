use image::GenericImageView;
use yaml_rust::YamlLoader;

///
/// Resources for a texture.
///

pub struct Texture {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub metadata: TextureMetadata,
    pub bind_group: wgpu::BindGroup
}

impl Texture {
    /// Creates atexture with the specified color.
    pub fn from_color(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layout: &wgpu::BindGroupLayout, name: &str, color: &wgpu::Color) -> Self {
        // Create bytes
        let bytes = [
            (color.b * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.r * 255.0) as u8,
            (color.a * 255.0) as u8
        ];
        
        // Create metadata
        let metadata = TextureMetadata {
            sprite_width: 1,
            sprite_height: 1
        };

        // Create texture
        return Texture::from_bytes(device, queue, bind_group_layout, name, &bytes, 1, 1, metadata);
    }

    /// Loads a texture from a path.
    pub fn from_path(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layout: &wgpu::BindGroupLayout, path: &str) -> Self {
        // Create path
        let true_path = std::path::Path::new(path);
        let name = true_path.file_stem().unwrap().to_str().unwrap().to_string();

        // Open image
        let tex_img = image::open(true_path).unwrap();

        // Load texture metadata
        let meta_path = true_path.with_extension("yaml");
        let mut metadata = TextureMetadata { 
            sprite_width: tex_img.width(),
            sprite_height: tex_img.height()
        };
        if meta_path.exists() {
            let meta_contents = std::fs::read_to_string(meta_path).unwrap();
            let doc = &YamlLoader::load_from_str(&meta_contents[..]).unwrap()[0];
            let sprite_width = doc["sprite_width"].as_i64().expect("Incorrect type of value for sprite_width in texture metadata.") as u32;
            let sprite_height = doc["sprite_width"].as_i64().expect("Incorrect type of value for sprite_width in texture metadata.") as u32;
            metadata = TextureMetadata {
                sprite_width,
                sprite_height,
            };
        }

        // Create texture
        return Texture::from_bytes(device, queue, bind_group_layout, &name[..], tex_img.to_bgra8().as_raw().as_slice(), tex_img.width(), tex_img.height(), metadata)
    }

    /// Creates a texture from a series of bytes.
    /// Bytes must represent a BGRA8 image.
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layout: &wgpu::BindGroupLayout, name: &str, bytes: &[u8], width: u32, height: u32, metadata: TextureMetadata) -> Self {
        let tex_size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Texture"),
                size: tex_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED
            }
        );

        // Write bytes from image to texture 
        queue.write_texture(
            wgpu::TextureCopyViewBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * tex_size.width,
                rows_per_image: tex_size.height
            },
            tex_size
        );

        // Create bind group
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(
                            &device.create_sampler(
                                & wgpu::SamplerDescriptor {
                                    label: Some("Sampler"),
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Nearest,
                                    min_filter: wgpu::FilterMode::Nearest,
                                    mipmap_filter: wgpu::FilterMode::Nearest,
                                    lod_min_clamp: 1.0,
                                    lod_max_clamp: 1.0,
                                    compare: None,
                                    anisotropy_clamp: None,
                                }
                            )
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(
                                &wgpu::TextureViewDescriptor {
                                    label: Some("Texture View"),
                                    format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
                                    dimension: Some(wgpu::TextureViewDimension::D2),
                                    aspect: wgpu::TextureAspect::All,
                                    base_mip_level: 0,
                                    level_count: None,
                                    base_array_layer: 0,
                                    array_layer_count: None,
                                }
                            )
                        )
                    }
                ],
            }
        );

        // Return struct
        Self {
            id: 0,
            name: String::from(name),
            width: tex_size.width,
            height: tex_size.height,
            metadata: metadata,
            bind_group,
        }
    }
}

/// Holds metadata for textures.
#[derive(Copy, Clone)]
pub struct TextureMetadata {
    pub sprite_width: u32,
    pub sprite_height: u32
}