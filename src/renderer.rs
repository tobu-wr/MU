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
    size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline
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
            present_mode: PresentMode::Mailbox
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);
        let vertex_shader_module = device.create_shader_module(include_spirv!("shader.vert.spv"));
        let fragment_shader_module = device.create_shader_module(include_spirv!("shader.frag.spv"));
        let pipeline_layout_descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);
        let vertex_stage_descriptor = ProgrammableStageDescriptor {
            module: &vertex_shader_module,
            entry_point: "main"
        };
        let fragment_stage_descriptor = ProgrammableStageDescriptor {
            module: &fragment_shader_module,
            entry_point: "main"
        };
        let rasterization_state_descriptor = RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false
        };
        let color_state_descriptor = ColorStateDescriptor {
            format: swap_chain_descriptor.format,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::ALL
        };
        let vertex_state_descriptor = VertexStateDescriptor {
            index_format: IndexFormat::Uint16,
            vertex_buffers: &[],
        };
        let render_pipeline_descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: vertex_stage_descriptor,
            fragment_stage: Some(fragment_stage_descriptor),
            rasterization_state: Some(rasterization_state_descriptor),
            color_states: &[color_state_descriptor],
            primitive_topology: PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: vertex_state_descriptor,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false
        };
        let render_pipeline = device.create_render_pipeline(&render_pipeline_descriptor);
        Self {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size,
            render_pipeline
        }
    }

    pub fn draw(&mut self, frame_buffer: &[u8]) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let encoder_descriptor = CommandEncoderDescriptor {
            label: None
        };
        let mut encoder = self.device.create_command_encoder(&encoder_descriptor);
        let render_pass_color_attachment_descriptor = RenderPassColorAttachmentDescriptor {
            attachment: &frame.view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true
            }
        };
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments: &[render_pass_color_attachment_descriptor],
            depth_stencil_attachment: None
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
