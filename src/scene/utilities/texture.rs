pub struct Texture {
	pub texture: wgpu::Texture,
	pub texture_view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl Texture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	pub fn create_depth_texture(
		device: &wgpu::Device,
		surface_configuration: &wgpu::SurfaceConfiguration,
		scene_name: &str,
	) -> Texture {
		let texture_extent = wgpu::Extent3d {
			width: surface_configuration.width,
			height: surface_configuration.height,
			depth_or_array_layers: 1,
		};
		let texture_descriptor = wgpu::TextureDescriptor {
			label: Some(&(scene_name.to_owned() + " depth texture")),
			size: texture_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		};
		let texture = device.create_texture(&texture_descriptor);
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some(&(scene_name.to_owned() + " sampler")),
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			compare: Some(wgpu::CompareFunction::LessEqual),
			..wgpu::SamplerDescriptor::default()
		});
		Self {
			texture,
			texture_view,
			sampler,
		}
	}
}
