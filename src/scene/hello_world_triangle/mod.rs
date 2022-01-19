use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct HelloWorldTriangleVertex {
	position: [f32; 2],
	color: [f32; 3],
}

pub struct HelloWorldTriangleScene {
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
}

impl HelloWorldTriangleScene {
	pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
		let shader_module = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Hello world triangle scene vertex buffer"),
			contents: bytemuck::cast_slice(&[
				HelloWorldTriangleVertex {
					position: [-0.5, -0.5],
					color: [1.0, 0.0, 0.0],
				},
				HelloWorldTriangleVertex {
					position: [0.5, -0.5],
					color: [0.0, 1.0, 0.0],
				},
				HelloWorldTriangleVertex {
					position: [0.0, 0.5],
					color: [0.0, 0.0, 1.0],
				},
			]),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Hello world triangle scene pipeline layout"),
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Hello world triangle scene pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vertex_stage",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<HelloWorldTriangleVertex>()
						as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
				entry_point: "fragment_stage",
				targets: &[wgpu::ColorTargetState {
					format: surface_configuration.format,
					blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::all(),
				}],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});
		Self {
			render_pipeline,
			vertex_buffer,
		}
	}
}

impl crate::scene::Scene for HelloWorldTriangleScene {
	fn resize(&mut self, _: &wgpu::Device, _: &wgpu::SurfaceConfiguration) {}

	fn update(&mut self, _: f32) {}

	fn render(
		&mut self,
		command_encoder: &mut wgpu::CommandEncoder,
		_: &wgpu::Queue,
		output_texture_view: &wgpu::TextureView,
	) {
		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Hello world triangle scene render pass"),
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view: output_texture_view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.5,
						g: 0.5,
						b: 0.5,
						a: 1.0,
					}),
					store: true,
				},
			}],
			depth_stencil_attachment: None,
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.draw(0..3, 0..1);
	}
}
