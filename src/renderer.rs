use wgpu::*;
use futures::executor::block_on;

use winit::{
    window::Window,
    dpi::PhysicalSize
};

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    swap_chain_descriptor: SwapChainDescriptor,
    swap_chain: SwapChain,
    size: PhysicalSize<u32>
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            compatible_surface: Some(&surface)
        };
        let adapter = block_on(instance.request_adapter(&adapter_options)).unwrap();
        let device_descriptor = DeviceDescriptor {
            features: Features::empty(),
            limits: Limits::default(),
            shader_validation: true
        };
        let (device, queue) = block_on(adapter.request_device(&device_descriptor, None)).unwrap();
        let swap_chain_descriptor = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo // TODO: test mailbox & immediate
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);
        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size
        }
    }

    pub fn draw(&mut self, frame_buffer: &[u8]) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let command_encoder_descriptor = CommandEncoderDescriptor {
            label: None
        };
        let mut encoder = self.device.create_command_encoder(&command_encoder_descriptor);
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments: &[
                RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: true
                    }
                }
            ],
            depth_stencil_attachment: None
        };
        {
            encoder.begin_render_pass(&render_pass_descriptor);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
