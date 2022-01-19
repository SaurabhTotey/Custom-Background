use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BouncingCubeVertex {
	position: [f32; 3],
}

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
	pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
		let shader_module = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene vertex buffer"),
			contents: bytemuck::cast_slice(&[
				// back face
				BouncingCubeVertex {
					position: [-0.1, -0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
				},
				// front face
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
				},
				// left face
				BouncingCubeVertex {
					position: [-0.1, -0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
				},
				// right face
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
				},
				// bottom face
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, -0.1, -0.1],
				},
				// top face
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
				},
			]),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene index buffer"),
			contents: &(0..6)
				.flat_map(|face| [0, 1, 2, 0, 2, 3].iter().map(|i| face * 4 + i))
				.collect::<Vec<_>>(),
			usage: wgpu::BufferUsages::INDEX,
		});
		let cube_transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene cube transform uniform buffer"),
			size: std::mem::size_of::<glam::Mat4>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		let cube_transform_uniform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Bouncing cube scene cube transform bind group layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});
		let cube_transform_uniform_bind_group =
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("Bouncing cube scene cube transform bind group"),
				layout: &cube_transform_uniform_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: cube_transform_uniform_buffer.as_entire_binding(),
				}],
			});
		let depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration,
			"Bouncing cube scene",
		);
		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Bouncing cube scene pipeline layout"),
				bind_group_layouts: &[&cube_transform_uniform_bind_group_layout],
				push_constant_ranges: &[],
			});
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Bouncing cube scene pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vertex_stage",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<BouncingCubeVertex>() as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3],
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
			depth_stencil: Some(wgpu::DepthStencilState {
				format: crate::scene::utilities::texture::Texture::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});
		Self {
			render_pipeline,
			vertex_buffer,
			index_buffer,
			cube_transform_uniform_buffer,
			cube_transform_uniform_bind_group,
			depth_texture,
		}
	}
}

impl crate::scene::Scene for BouncingCubeScene {
	fn resize(
		&mut self,
		device: &wgpu::Device,
		surface_configuration: &wgpu::SurfaceConfiguration,
	) {
		self.depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration,
			"Bouncing cube scene",
		);
	}

	fn update(&mut self, dt: f32) {
		self.cube_position += self.cube_velocity * dt;
		// TODO: I am assuming a cube of sidelength 0.2 with the position defining its center
		// TODO: Need to look into pushing z back to higher values with a transform possibly
		if self.cube_position.x < -0.9 {
			self.cube_position.x = -0.9;
			self.cube_position.x *= -1.0;
		}
		if self.cube_position.x > 0.9 {
			self.cube_position.x = 0.9;
			self.cube_position.x *= -1.0;
		}
		if self.cube_position.y < -0.9 {
			self.cube_position.y = -0.9;
			self.cube_position.y *= -1.0;
		}
		if self.cube_position.y > 0.9 {
			self.cube_position.y = 0.9;
			self.cube_position.y *= -1.0;
		}
		if self.cube_position.z < -0.9 {
			self.cube_position.z = -0.9;
			self.cube_position.z *= -1.0;
		}
		if self.cube_position.z > 0.9 {
			self.cube_position.z = 0.9;
			self.cube_position.z *= -1.0;
		}
	}

	fn render(
		&mut self,
		command_encoder: &mut wgpu::CommandEncoder,
		output_texture_view: &wgpu::TextureView,
	) {
		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Bouncing cube scene render pass"),
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
		// TODO: update cube transform
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.set_bind_group(0, &self.cube_transform_uniform_bind_group, &[]);
		// TODO: draw call
	}
}