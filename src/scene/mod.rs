pub mod hello_world_triangle;
pub mod utilities;

/**
 * List required functionality of all scenes.
 */
pub trait Scene {
	fn resize(&mut self, _: &wgpu::SurfaceConfiguration);
	fn update(&mut self, _: f32);
	fn render(&mut self, _: &mut wgpu::CommandEncoder, _: &wgpu::TextureView);
}
