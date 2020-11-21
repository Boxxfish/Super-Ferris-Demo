///
/// Renderer that uses WGPU to render quads.
/// Quads are rendered with pixel level precision.
///

use winit::window;
use bytemuck;
use wgpu::util::DeviceExt;
use wgpu;
use image::GenericImageView;

pub struct Renderer {
    swap_chain: wgpu::SwapChain,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    device: wgpu::Device,
    render_pipeline: wgpu::RenderPipeline,
    tex_bind_group_layout: wgpu::BindGroupLayout,
    per_quad_bind_group_layout: wgpu::BindGroupLayout,
    glob_bind_group: wgpu::BindGroup,
    textures: Vec<Texture>,
    render_quads: Vec<RenderQuad>
}

pub const WIN_SCALE: u32 = 2;
pub const WIN_WIDTH: u32 = 256 * WIN_SCALE;
pub const WIN_HEIGHT: u32 = 240 * WIN_SCALE;

impl Renderer {
    /// Creates a new renderer instance.
    pub async fn new(win: &window::Window) -> Self {
        // Set up WGPU stuff
        let wgpu_inst = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { wgpu_inst.create_surface(win) };
        let adapter = wgpu_inst.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface)
        }).await.expect("Could not create WGPU adapter.");
        let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                shader_validation: true 
            }, 
            None
        ).await.expect("Could not create WGPU device.");
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            present_mode: wgpu::PresentMode::Fifo
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        // Load shaders
        let v_shader = Renderer::load_shader(&device, include_bytes!("shaders/basic.vert.spirv"));
        let f_shader = Renderer::load_shader(&device, include_bytes!("shaders/basic.frag.spirv"));

        // Create bind groups
        let tex_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                            multisampled: false,
                        },
                        count: None
                    }
                ]
            }
        );

        let glob_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Global Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None
                    }
                ]
            }
        );

        let per_quad_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Per Quad Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None
                    }
                ]
            }
        );

        // Create global uniform bind group
        let glob_uniforms = GlobalUniforms::default();
        let glob_uniform_buff = device.create_buffer_init(
           &wgpu::util::BufferInitDescriptor {
               label: Some("Global Uniform Buffer"),
               contents: bytemuck::cast_slice(&[glob_uniforms]),
               usage: wgpu::BufferUsage::UNIFORM,
           } 
        );
        let glob_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Global Bind Group Descriptor"),
                layout: &glob_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(glob_uniform_buff.slice(..)),
                    }
                ]
            }
        );

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&tex_bind_group_layout, &glob_bind_group_layout, &per_quad_bind_group_layout],
                push_constant_ranges: &[],
            }
        );
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &v_shader,
                    entry_point: "main"
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &f_shader,
                    entry_point: "main"
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    clamp_depth: false,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[
                    wgpu::ColorStateDescriptor {
                        format: swap_chain_desc.format,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[
                        wgpu::VertexBufferDescriptor {
                            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::InputStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttributeDescriptor {
                                    offset: 0,
                                    format: wgpu::VertexFormat::Float3,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttributeDescriptor {
                                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                    format: wgpu::VertexFormat::Float2,
                                    shader_location: 1,
                                }
                            ],
                        }
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false
            }
        );

        // Create texture list, with an empty black texture
        let black_tex = Texture::from_color(&device, &queue, &tex_bind_group_layout, "black", &wgpu::Color::BLACK);
        let textures = vec![black_tex];

        // Return struct
        Self {
            swap_chain,
            queue,
            surface,
            device,
            render_pipeline,
            tex_bind_group_layout,
            per_quad_bind_group_layout,
            glob_bind_group,
            textures,
            render_quads: Vec::new()
        }
    }

    /// Creates a render quad and returns its ID.
    pub fn create_render_quad(&mut self) -> u32 {
        let r_quad = RenderQuad::new(&self.device, &self.per_quad_bind_group_layout);
        self.render_quads.push(r_quad);

        return self.render_quads.len() as u32 - 1;
    }

    /// Loads a texture and returns its ID.
    /// May be already cached.
    pub fn load_texture(&mut self, tex_name: &str) -> u32 {
        match self.textures.iter().find(|x| x.name == tex_name) {
            Some(x) => {x.id},
            None => {
                let mut tex = Texture::from_path(&self.device, &self.queue, &self.tex_bind_group_layout, tex_name);
                let tex_id = self.textures.len() as u32;
                tex.id = tex_id;
                self.textures.push(tex);
                tex_id
            }
        }
    }

    /// Attaches a texture to a render quad.
    /// If width or height are different, regenerates quad.
    pub fn attach_tex_to_quad(&mut self, quad_id: u32, tex_id: u32) {
        // Regenerate vertex buffer if necessary
        let old_tex = &self.textures[self.render_quads[quad_id as usize].tex_id as usize];
        let new_tex = &self.textures[tex_id as usize];
        if old_tex.width != new_tex.width || old_tex.height != new_tex.height {
            self.render_quads[quad_id as usize].vertex_buffer = RenderQuad::gen_vertex_buffer(&self.device, new_tex.width, new_tex.height);
        }

        self.render_quads[quad_id as usize].tex_id = tex_id;
    }

    /// Sets the position of a render quad.
    pub fn set_quad_pos(&mut self, quad_id: u32, x: u32, y: u32) {
        let matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::new((x * WIN_SCALE) as f32, (y * WIN_SCALE) as f32, 0.0));
        self.render_quads[quad_id as usize].per_quad_bind_group = RenderQuad::gen_per_quad_bind_group(&self.device, &self.per_quad_bind_group_layout, matrix);
    }

    /// Renders all render quads.
    pub fn render(&mut self) {
        // Get next frame from swap chain
        let next_frame = self.swap_chain.get_current_frame().expect("Could not retrieve frame from swap chain.");
        
        // Create the next render pass
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder")
            }
        );
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &next_frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0
                                }
                            ),
                            store: true
                        }
                    },
                ],
                depth_stencil_attachment: None
            }
        );

        render_pass.set_pipeline(&self.render_pipeline);

        for render_quad in self.render_quads.as_slice() {
            render_pass.set_bind_group(0, &self.textures[render_quad.tex_id as usize].bind_group, &[]);
            render_pass.set_bind_group(1, &self.glob_bind_group, &[]);
            render_pass.set_bind_group(2, &render_quad.per_quad_bind_group, &[]);
            render_pass.set_vertex_buffer(0, render_quad.vertex_buffer.slice(..));
            render_pass.draw(0..QUAD_V_SIZE as u32, 0..1);
        }

        // Submit rendering commands to the queue
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Loads a shader.
    fn load_shader(device: &wgpu::Device, bytes: &[u8]) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::util::make_spirv(bytes))
    }
}

/// A quad to be rendered.
pub struct RenderQuad {
    vertex_buffer: wgpu::Buffer,
    tex_id: u32,
    per_quad_bind_group: wgpu::BindGroup
}

const QUAD_V_SIZE: u32 = 6;

impl RenderQuad {
    /// Creates a new render quad.
    pub fn new(device: &wgpu::Device, per_quad_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        Self {
            vertex_buffer: RenderQuad::gen_vertex_buffer(device, 0, 0),
            tex_id: 0,
            per_quad_bind_group: RenderQuad::gen_per_quad_bind_group(device, per_quad_bind_group_layout, cgmath::Matrix4::from_scale(1.0)),
        }
    }

    /// Generates a vertex buffer.
    fn gen_vertex_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
        // Create new coordinates
        let quad_width = (width * WIN_SCALE) as f32;
        let quad_height = (height * WIN_SCALE) as f32;
        let quad_coords: &[Vertex] = &[
            Vertex { position: [0.0, quad_height, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [quad_width as f32, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [quad_width, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [quad_width, quad_height, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.0, quad_height, 0.0], tex_coords: [0.0, 1.0] },
        ];

        // Generate vertex buffer
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(quad_coords),
                usage: wgpu::BufferUsage::VERTEX
            }
        );

        return vertex_buffer;
    }

    /// Generates a per quad binding group.
    fn gen_per_quad_bind_group(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, matrix: cgmath::Matrix4<f32>) -> wgpu::BindGroup {
        let mut uniform_struct = PerQuadUniforms::default();
        uniform_struct.model_mat = matrix;

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Per Quad Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform_struct]),
                usage: wgpu::BufferUsage::UNIFORM,
            }
        );

        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Per Quad Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                    }
                ],
            }
        )
    }
}

/// Stores vertex and attributes.
#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2]
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

/// Resources for a texture.
pub struct Texture {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
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
        
        // Create texture
        return Texture::from_bytes(device, queue, bind_group_layout, name, &bytes, 1, 1);
    }

    /// Loads a texture from a path.
    pub fn from_path(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layout: &wgpu::BindGroupLayout, path: &str) -> Self {
        // Create path
        let true_path = std::path::Path::new(path);
        let name = true_path.file_stem().unwrap().to_str().unwrap().to_string();

        // Open image
        let tex_img = image::open(true_path).unwrap();

        // Create texture
        return Texture::from_bytes(device, queue, bind_group_layout, &name[..], tex_img.to_bgra8().as_raw().as_slice(), tex_img.width(), tex_img.height())
    }

    /// Creates a texture from a series of bytes.
    /// Bytes must represent a BGRA8 image.
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bind_group_layout: &wgpu::BindGroupLayout, name: &str, bytes: &[u8], width: u32, height: u32) -> Self {
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
            bind_group,
        }
    }
}

/// Holds global uniforms.
#[derive(Copy, Clone)]
struct GlobalUniforms {
    proj_mat: cgmath::Matrix4<f32>
}

unsafe impl bytemuck::Pod for GlobalUniforms {}
unsafe impl bytemuck::Zeroable for GlobalUniforms {}

impl GlobalUniforms {
    /// Returns the default global uniform struct.
    pub fn default() -> Self {
        let win_width = WIN_WIDTH as f32;
        let win_height = WIN_HEIGHT as f32;
        let proj_mat = cgmath::ortho(
            0.0, 
            win_width, 
            win_height, 
            0.0, 
            -1.0, 
            1.0
        );
        Self {
            proj_mat
        }
    }
}

/// Holds per quad uniforms.
#[derive(Copy, Clone)]
struct PerQuadUniforms {
    model_mat: cgmath::Matrix4<f32>
}

unsafe impl bytemuck::Pod for PerQuadUniforms {}
unsafe impl bytemuck::Zeroable for PerQuadUniforms {}

impl PerQuadUniforms {
    /// Returns the default per quad uniform struct.
    pub fn default() -> Self {
        let model_mat = cgmath::Matrix4::from_scale(1.0);
        Self {
            model_mat
        }
    }
}