use wgpu::util::DeviceExt;

pub struct BouncingCubeScene {
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	cube_transform_uniform_buffer: wgpu::Buffer,
	cube_transform_uniform_bind_group: wgpu::BindGroup,
	depth_texture: crate::scene::utilities::texture::Texture,
	camera: crate::scene::utilities::camera::Camera,
	cube_position: glam::Vec3A,
	cube_velocity: glam::Vec3A,
}

impl BouncingCubeScene {

}

impl crate::scene::Scene for BouncingCubeScene {
	fn resize(&mut self, _: &wgpu::SurfaceConfiguration) {}
	fn update(&mut self, _: f32) {}
	fn render(
		&mut self,
		command_encoder: &mut wgpu::CommandEncoder,
		output_texture_view: &wgpu::TextureView,
	) {
		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Bouncing cube render pass"),
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
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &self.depth_texture.texture_view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: true,
				}),
				stencil_ops: None,
			}),
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.set_bind_group(0, &self.cube_transform_uniform_bind_group, &[]);
		render_pass.draw(0..3, 0..1);
	}
}
