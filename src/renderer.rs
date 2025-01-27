///
/// Renderer that uses WGPU to render quads.
/// Quads are rendered with pixel level precision.
///

use winit::window;
use bytemuck;
use wgpu::util::DeviceExt;
use wgpu;
use image::GenericImageView;
use yaml_rust::{YamlLoader, YamlEmitter};
use crate::texture::Texture;

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
            self.render_quads[quad_id as usize].vertex_buffer = RenderQuad::gen_vertex_buffer(&self.device, new_tex.width, new_tex.height, 0.0, 1.0, 1.0, 0.0);
        }

        self.render_quads[quad_id as usize].tex_id = tex_id;
    }

    /// Attaches a texture as a sprite to a render quad.
    /// This always regenerates the vertex buffer, for now.
    pub fn attach_sprite_to_quad(&mut self, quad_id: u32, tex_id: u32, sprite_index: u32) {
        // Generate vertex buffer with correct texture coordinates
        let new_tex = &self.textures[tex_id as usize];
        let sprite_width = new_tex.metadata.sprite_width;
        let sprite_height = new_tex.metadata.sprite_height;
        let sprites_per_row = new_tex.width / sprite_width;
        let y = sprite_index / sprites_per_row;
        let x = sprite_index - y * sprites_per_row;
        let tex_coord_width = sprite_width as f32 / new_tex.width as f32;
        let tex_coord_height = sprite_height as f32 / new_tex.height as f32;
        let sprite_l= x as f32 * tex_coord_width;
        let sprite_r = (x + 1) as f32 * tex_coord_width;
        let sprite_t = (y + 1) as f32 * tex_coord_height;
        let sprite_b = y as f32 * tex_coord_height;
        self.render_quads[quad_id as usize].vertex_buffer = RenderQuad::gen_vertex_buffer(&self.device, sprite_width, sprite_height, sprite_l, sprite_r, sprite_t, sprite_b);

        self.render_quads[quad_id as usize].tex_id = tex_id;
    }

    /// Attaches a texture as a sprite to a render quad as tilemap source.
    /// This always regenerates the vertex buffer, for now.
    pub fn attach_tilemap_to_quad(&mut self, quad_id: u32, tex_id: u32, tilemap: &[u32], tilemap_width: u32, tilemap_height: u32) {
        // Generate vertex buffer with correct texture coordinates
        let new_tex = &self.textures[tex_id as usize];
        self.render_quads[quad_id as usize].vertex_buffer = RenderQuad::gen_tilemap_vertex_buffer(&self.device, tilemap, tilemap_width, new_tex);
        self.render_quads[quad_id as usize].vertex_count = tilemap_width * tilemap_height * QUAD_V_SIZE;
        self.render_quads[quad_id as usize].tex_id = tex_id;
    }

    /// Sets the position of a render quad.
    pub fn set_quad_pos(&mut self, quad_id: u32, x: i32, y: i32) {
        let matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::new((x * WIN_SCALE as i32) as f32, (y * WIN_SCALE as i32) as f32, 0.0));
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
            render_pass.draw(0..render_quad.vertex_count, 0..1);
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
    vertex_count: u32,
    tex_id: u32,
    per_quad_bind_group: wgpu::BindGroup
}

const QUAD_V_SIZE: u32 = 6;

impl RenderQuad {
    /// Creates a new render quad.
    pub fn new(device: &wgpu::Device, per_quad_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        Self {
            vertex_buffer: RenderQuad::gen_vertex_buffer(device, 0, 0, 0.0, 1.0, 1.0, 0.0),
            tex_id: 0,
            vertex_count: QUAD_V_SIZE,
            per_quad_bind_group: RenderQuad::gen_per_quad_bind_group(device, per_quad_bind_group_layout, cgmath::Matrix4::from_scale(1.0)),
        }
    }

    /// Generates a vertex buffer.
    fn gen_vertex_buffer(device: &wgpu::Device, width: u32, height: u32, sprite_l: f32, sprite_r: f32, sprite_t: f32, sprite_b: f32) -> wgpu::Buffer {
        // Create new coordinates
        let quad_width = (width * WIN_SCALE) as f32;
        let quad_height = (height * WIN_SCALE) as f32;
        let quad_coords: &[Vertex] = &[
            Vertex { position: [0.0, quad_height, 0.0], tex_coords: [sprite_l, sprite_t] },
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [sprite_l, sprite_b] },
            Vertex { position: [quad_width as f32, 0.0, 0.0], tex_coords: [sprite_r, sprite_b] },
            Vertex { position: [quad_width, 0.0, 0.0], tex_coords: [sprite_r, sprite_b] },
            Vertex { position: [quad_width, quad_height, 0.0], tex_coords: [sprite_r, sprite_t] },
            Vertex { position: [0.0, quad_height, 0.0], tex_coords: [sprite_l, sprite_t] },
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

    /// Generates a vertex buffer for a tilemap.
    fn gen_tilemap_vertex_buffer(device: &wgpu::Device, tilemap: &[u32], tilemap_width: u32, texture: &Texture) -> wgpu::Buffer {
        // Create new coordinates
        let mut quad_coords = Vec::new();
        let quad_width = (texture.metadata.sprite_width * WIN_SCALE) as f32;
        let quad_height = (texture.metadata.sprite_height * WIN_SCALE) as f32;
        let mut x = 0;
        let mut y = 0;
        for tile in tilemap {
            let sprites_per_row = texture.width / texture.metadata.sprite_width;
            let sprite_y = tile / sprites_per_row;
            let sprite_x = tile - sprite_y * sprites_per_row;
            let tex_coord_width = texture.metadata.sprite_width as f32 / texture.width as f32;
            let tex_coord_height = texture.metadata.sprite_height as f32 / texture.height as f32;
            let sprite_l= sprite_x as f32 * tex_coord_width;
            let sprite_r = (sprite_x + 1) as f32 * tex_coord_width;
            let sprite_t = (sprite_y + 1) as f32 * tex_coord_height;
            let sprite_b = sprite_y as f32 * tex_coord_height;
            let x_coord = (x as f32) * quad_width;
            let y_coord = (y as f32) * quad_height;
            quad_coords.append(&mut vec![
                Vertex { position: [x_coord + 0.0, y_coord + quad_height, 0.0], tex_coords: [sprite_l, sprite_t] },
                Vertex { position: [x_coord + 0.0, y_coord + 0.0, 0.0], tex_coords: [sprite_l, sprite_b] },
                Vertex { position: [x_coord + quad_width, y_coord + 0.0, 0.0], tex_coords: [sprite_r, sprite_b] },
                Vertex { position: [x_coord + quad_width, y_coord + 0.0, 0.0], tex_coords: [sprite_r, sprite_b] },
                Vertex { position: [x_coord + quad_width, y_coord + quad_height, 0.0], tex_coords: [sprite_r, sprite_t] },
                Vertex { position: [x_coord + 0.0, y_coord + quad_height, 0.0], tex_coords: [sprite_l, sprite_t] },
            ]);
            x += 1;
            if x == tilemap_width {
                x = 0;
                y += 1;
            }
        };

        // Generate vertex buffer
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(quad_coords.as_slice()),
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