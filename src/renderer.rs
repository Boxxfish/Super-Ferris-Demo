///
/// Renderer that uses WGPU to render quads.
/// Quads are rendered with pixel level precision.
///

use winit::window;
use winit::event_loop;

use wgpu;

pub struct Renderer {
    swap_chain: wgpu::SwapChain,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    device: wgpu::Device,
    render_quads: Vec<RenderQuad>
}

pub const WIN_SCALE: u32 = 2;
pub const WIN_WIDTH: u32 = 256 * WIN_SCALE;
pub const WIN_HEIGHT: u32 = 240 * WIN_SCALE;

impl Renderer {
    /// Creates a new renderer instance.
    pub async fn new(win: &window::Window) -> Self {
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

        Self {
            swap_chain,
            queue,
            surface,
            device,
            render_quads: Vec::new()
        }
    }

    /// Creates a render quad and returns its ID.
    pub fn create_render_quad(&mut self) -> u32 {
        todo!()
    }

    /// Loads a texture and returns its ID.
    /// May be already cached.
    pub fn load_texture(&mut self, tex_name: &str) -> u32 {
        todo!()
    }

    /// Renders all render quads.
    pub fn render(&mut self) {
        let next_frame = self.swap_chain.get_current_frame().expect("Could not retrieve frame from swap chain.");
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder")
            }
        );
        let render_pass = encoder.begin_render_pass(
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
                    }
                ],
                depth_stencil_attachment: None
            }
        );
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

/// Window resources.
pub struct RendererWindow {
    pub window: window::Window,
    pub event_loop: event_loop::EventLoop<()>
}

/// A quad to be rendered.
pub struct RenderQuad {
    pub order: i32,
    pub tex_id: u32,
    pub x: i32,
    pub y: i32
}

/// Resources for a texture.
pub struct Texture {
    pub name: String
}