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
    render_pipeline: RenderPipeline,
    texture: Texture,
    bind_group: BindGroup
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
        let texture_size = Extent3d {
            width: 256,
            height: 240,
            depth: 1
        };
        let texture_descriptor = TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: None
        };
        let texture = device.create_texture(&texture_descriptor);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler_descriptor = SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);
        let bind_group_layout_entry_0 = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::SampledTexture {
                multisampled: false,
                dimension: TextureViewDimension::D2,
                component_type: TextureComponentType::Uint
            },
            count: None
        };
        let bind_group_layout_entry_1 = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Sampler {
                comparison: false
            },
            count: None
        };
        let bind_group_layout_descriptor = BindGroupLayoutDescriptor {
            entries: &[bind_group_layout_entry_0, bind_group_layout_entry_1],
            label: None
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
        let bind_group_entry_0 = BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&texture_view)
        };
        let bind_group_entry_1 = BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(&sampler)
        };
        let bind_group_descriptor = BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[bind_group_entry_0, bind_group_entry_1],
            label: None
        };
        let bind_group = device.create_bind_group(&bind_group_descriptor);
        let vertex_shader_module = device.create_shader_module(include_spirv!("shader.vert.spv"));
        let fragment_shader_module = device.create_shader_module(include_spirv!("shader.frag.spv"));
        let pipeline_layout_descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
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
            render_pipeline,
            texture,
            bind_group
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
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
        let texture_copy_view = TextureCopyView {
            texture: &self.texture,
            mip_level: 0,
            origin: Origin3d::ZERO
        };
        let texture_data_layout = TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * 256,
            rows_per_image: 240,
        };
        self.queue.write_texture(texture_copy_view, frame_buffer, texture_data_layout,
            Extent3d {
            width: 256,
            height: 240,
            depth: 1
        });
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
