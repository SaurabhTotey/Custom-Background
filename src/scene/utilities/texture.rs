pub struct Texture {
	pub texture: wgpu::Texture,
	pub texture_view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
	pub sample_type: wgpu::TextureSampleType,
	pub view_dimension: wgpu::TextureViewDimension,
	pub sampler_binding_type: wgpu::SamplerBindingType,
}

impl Texture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	pub fn create_depth_texture(
		device: &wgpu::Device,
		width: u32,
		height: u32,
		scene_name: &str,
	) -> Texture {
		let texture_extent = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		let texture_descriptor_label = &(scene_name.to_owned() + " depth texture");
		let texture_descriptor = wgpu::TextureDescriptor {
			label: Some(texture_descriptor_label),
			size: texture_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT
				| wgpu::TextureUsages::TEXTURE_BINDING
				| wgpu::TextureUsages::COPY_SRC,
		};
		let texture = device.create_texture(&texture_descriptor);
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some(&(scene_name.to_owned() + " sampler")),
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			..wgpu::SamplerDescriptor::default()
		});
		Self {
			texture,
			texture_view,
			sampler,
			sample_type: wgpu::TextureSampleType::Depth,
			view_dimension: wgpu::TextureViewDimension::D2,
			sampler_binding_type: wgpu::SamplerBindingType::Comparison,
		}
	}

	pub fn create_bind_group(
		&self,
		device: &wgpu::Device,
		label: &str,
		visibility: wgpu::ShaderStages,
	) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
		let label = label.to_owned() + " bind group";
		let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some(&(label.clone() + " layout")),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility,
					ty: wgpu::BindingType::Texture {
						sample_type: self.sample_type,
						view_dimension: self.view_dimension,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility,
					ty: wgpu::BindingType::Sampler(self.sampler_binding_type),
					count: None,
				},
			],
		});
		let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&label),
			layout: &layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&self.texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&self.sampler),
				},
			],
		});
		(layout, group)
	}
}
