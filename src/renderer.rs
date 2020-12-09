// WIP
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
    swap_chain_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let request_adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            compatible_surface: Some(&surface)
        };
        let adapter = block_on(instance.request_adapter(&request_adapter_options)).unwrap();
        let device_desc = DeviceDescriptor {
            features: Features::empty(),
            limits: Limits::default(),
            shader_validation: true
        };
        let (device, queue) = block_on(adapter.request_device(&device_desc, None)).unwrap();
        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
        //============================
        let texture = device.create_texture(
            &TextureDescriptor {
                // All textures are stored as 3d, we represent our 2d texture
                // by setting depth to 1.
                size: Extent3d {
                    width: 256,
                    height: 240,
                    depth: 1,
                },
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                // SAMPLED tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
                label: Some("texture")
            }
        );
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            multisampled: false,
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler {
                            comparison: false,
                        },
                        count: None,
                    },
                ],
                label: Some("bind_group_layout")
            }
        );
        let bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("bind_group")
            }
        );
        // =============================================
        let vs_src = include_str!("shader.vert");
        let fs_src = include_str!("shader.frag");
        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler.compile_into_spirv(vs_src, shaderc::ShaderKind::Vertex, "shader.vert", "main", None).unwrap();
        let fs_spirv = compiler.compile_into_spirv(fs_src, shaderc::ShaderKind::Fragment, "shader.frag", "main", None).unwrap();
        let vs_module = device.create_shader_module(util::make_spirv(&vs_spirv.as_binary_u8()));
        let fs_module = device.create_shader_module(util::make_spirv(&fs_spirv.as_binary_u8()));
        let pipeline_layout_desc = PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main", // 1.
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor { // 2.
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(
                RasterizationStateDescriptor {
                    front_face: FrontFace::Ccw,
                    cull_mode: CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                }
            ),
            color_states: &[
                ColorStateDescriptor {
                    format: swap_chain_desc.format,
                    color_blend: BlendDescriptor::REPLACE,
                    alpha_blend: BlendDescriptor::REPLACE,
                    write_mask: ColorWrite::ALL,
                },
            ],
            primitive_topology: PrimitiveTopology::TriangleList, // 1.
            depth_stencil_state: None, // 2.
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16, // 3.
                vertex_buffers: &[], // 4.
            },
            sample_count: 1, // 5.
            sample_mask: !0, // 6.
            alpha_to_coverage_enabled: false, // 7.
        });
        //===============================
        
        Self {
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            render_pipeline,
            texture,
            texture_view,
            sampler,
            bind_group
        }
    }

  /*  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }*/

    pub fn draw(&mut self, frame_buffer: &[u8]) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let command_encoder_desc = CommandEncoderDescriptor {
            label: Some("Render Encoder")
        };
        
        let mut encoder = self.device.create_command_encoder(&command_encoder_desc);

        let size = wgpu::Extent3d {
            width: 256,
            height: 240,
            depth: 1,
        };
        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            // The actual pixel data
            &frame_buffer,
            // The layout of the texture
            TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * 256,
                rows_per_image: 240,
            },
           size,
        );



        let render_pass_color_attachment_desc = RenderPassColorAttachmentDescriptor {
            attachment: &frame.view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true
            }
        };
        let render_pass_desc = RenderPassDescriptor {
            color_attachments: &[render_pass_color_attachment_desc],
            depth_stencil_attachment: None
        };

        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc); // 1.
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]); 
            render_pass.draw(0..3, 0..1); 
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
