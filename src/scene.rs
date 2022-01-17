/**
 * TODO: eventually make a generalized trait for Scenes that specific scenes can implement
 */
pub struct HelloWorldTriangleScene {
	render_pipeline: wgpu::RenderPipeline,
}

impl HelloWorldTriangleScene {

	pub fn new(device: &wgpu::Device) -> Self {
		let shader_module = device.create_shader_module(&wgpu::include_wgsl!("hello_world_triangle.wgsl"));
		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
				buffers: &[]
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
				entry_point: "fragment_stage",
				targets: &[
					wgpu::ColorTargetState {
						format: wgpu::TextureFormat::Bgra8Unorm, // TODO: don't hardcode this
						blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::all(),
					}
				],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});
		Self {
			render_pipeline
		}
	}

	pub fn render(&self, command_encoder: &mut wgpu::CommandEncoder, output_texture_view: &wgpu::TextureView) {
		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Default render pass"),
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view: output_texture_view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 72.0 / 255.0,
						g: 56.0 / 255.0,
						b: 98.0 / 255.0,
						a: 1.0,
					}),
					store: true,
				},
			}],
			depth_stencil_attachment: None,
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.draw(0..3, 0..1);
	}

}
