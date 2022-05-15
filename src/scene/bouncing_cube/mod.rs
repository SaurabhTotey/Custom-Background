mod bouncing_cube_model;
use wgpu::util::DeviceExt;

/**
 * TODO:
 *  * point lights with cube shadows on wall
 *  * blinn-phong lighting
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	normal: [f32; 3],
	color: [f32; 3],
}

#[repr(C, align(256))]
#[derive(Clone, Copy, bytemuck::Zeroable)]
struct InstanceTransform {
	matrix: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightInformationDatum {
	position: [f32; 3],
	_padding_0: u32,
	diffuse_color: [f32; 3],
	_padding_1: u32,
}

pub struct BouncingCubeScene {
	bouncing_cube_model: bouncing_cube_model::BouncingCubeSceneInformation,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	render_camera_uniform_buffer: wgpu::Buffer,
	render_camera_bind_group: wgpu::BindGroup,
	instance_model_dynamic_uniform_buffer: wgpu::Buffer,
	instance_normal_dynamic_uniform_buffer: wgpu::Buffer,
	instance_bind_group: wgpu::BindGroup,
	instance_buffer_spacing: wgpu::BufferAddress,
	light_information_buffer: wgpu::Buffer,
	light_information_bind_group: wgpu::BindGroup,
	depth_texture: crate::scene::utilities::texture::Texture,
}

impl BouncingCubeScene {
	pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
		// Make the model that this scene represents.
		let bouncing_cube_model = bouncing_cube_model::BouncingCubeSceneInformation::new(
			surface_configuration.width as f32,
			surface_configuration.height as f32,
		);

		// Get shader.
		let render_shader_module = device.create_shader_module(&wgpu::include_wgsl!("render.wgsl"));

		// Create buffers and bind groups.
		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene vertex buffer"),
			size: std::mem::size_of::<Vertex>() as wgpu::BufferAddress * 44, // 24 for cube, 20 for walls
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene index buffer"),
			contents: bytemuck::cast_slice(
				&(0..11) // 6 cube faces and 5 walls
					.flat_map(|face| [0u16, 1, 2, 0, 2, 3].iter().map(move |i| face * 4 + i))
					.collect::<Vec<_>>(),
			),
			usage: wgpu::BufferUsages::INDEX,
		});
		let (
			render_camera_uniform_buffer,
			render_camera_bind_group_layout,
			render_camera_bind_group,
		) = bouncing_cube_model
			.scene_camera
			.create_bind_group(device, "Bouncing cube scene");
		let depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration,
			"Bouncing cube scene",
		);

		// Create dynamic uniform buffer for instance data.
		let instance_buffer_spacing =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let instance_model_dynamic_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene instance model dynamic uniform buffer"),
			size: 11 * instance_buffer_spacing, // 11 quads each with their own transform
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
			mapped_at_creation: false,
		});
		let instance_normal_dynamic_uniform_buffer =
			device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("Bouncing cube scene instance normal dynamic uniform buffer"),
				size: 11 * instance_buffer_spacing,
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
				mapped_at_creation: false,
			});
		let instance_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Bouncing cube scene instance bind group layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: true,
							min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
								InstanceTransform,
							>() as _),
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: true,
							min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
								InstanceTransform,
							>() as _),
						},
						count: None,
					},
				],
			});
		let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Bouncing cube scene instance bind group"),
			layout: &instance_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &instance_model_dynamic_uniform_buffer,
						offset: 0,
						size: wgpu::BufferSize::new(std::mem::size_of::<InstanceTransform>() as _),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &instance_normal_dynamic_uniform_buffer,
						offset: 0,
						size: wgpu::BufferSize::new(std::mem::size_of::<InstanceTransform>() as _),
					}),
				},
			],
		});

		// Create uniform buffer for light information.
		let light_information_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Boucning cube scene light information uniform buffer"),
			size: 3 * std::mem::size_of::<LightInformationDatum>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
			mapped_at_creation: false,
		});
		let light_information_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Bouncing cube scene light information bind group layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});
		let light_information_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Bouncing cube scene light information bind group"),
			layout: &light_information_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: light_information_buffer.as_entire_binding(),
			}],
		});

		// Create pipeline layout and pipeline.
		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Bouncing cube scene pipeline layout"),
				bind_group_layouts: &[
					&render_camera_bind_group_layout,
					&instance_bind_group_layout,
					&light_information_bind_group_layout,
				],
				push_constant_ranges: &[wgpu::PushConstantRange {
					stages: wgpu::ShaderStages::FRAGMENT,
					range: 0..12, // 12 bytes for a vector of 3 floats (and each float is 4 bytes)
				}],
			});
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Bouncing cube scene pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &render_shader_module,
				entry_point: "vertex_stage",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &render_shader_module,
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
			bouncing_cube_model,
			render_pipeline,
			vertex_buffer,
			index_buffer,
			render_camera_uniform_buffer,
			render_camera_bind_group,
			instance_model_dynamic_uniform_buffer,
			instance_normal_dynamic_uniform_buffer,
			instance_bind_group,
			instance_buffer_spacing,
			light_information_buffer,
			light_information_bind_group,
			depth_texture,
		}
	}

	// Vertices for the cube in its own reference frame: all need to be transformed into world coordinates.
	const CUBE_VERTICES: [Vertex; 24] = [
		// back face
		Vertex {
			position: [-1.0, -1.0, -1.0],
			normal: [0.0, 0.0, -1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [1.0, -1.0, -1.0],
			normal: [0.0, 0.0, -1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [1.0, 1.0, -1.0],
			normal: [0.0, 0.0, -1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [-1.0, 1.0, -1.0],
			normal: [0.0, 0.0, -1.0],
			color: [1.0, 0.0, 0.0],
		},
		// front face
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [0.0, 0.0, 1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [0.0, 0.0, 1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [0.0, 0.0, 1.0],
			color: [1.0, 0.0, 0.0],
		},
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [0.0, 0.0, 1.0],
			color: [1.0, 0.0, 0.0],
		},
		// left face
		Vertex {
			position: [-1.0, -1.0, -1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [-1.0, 1.0, -1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		// right face
		Vertex {
			position: [1.0, -1.0, -1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		Vertex {
			position: [1.0, 1.0, -1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.0, 1.0, 0.0],
		},
		// bottom face
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [1.0, -1.0, -1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [-1.0, -1.0, -1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		// top face
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [1.0, 1.0, -1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
		Vertex {
			position: [-1.0, 1.0, -1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.0, 0.0, 1.0],
		},
	];

	// Vertices for the walls in their own reference frame: all need to be transformed into world coordinates (only scaled).
	const WALL_COORDINATES: [Vertex; 20] = [
		// back wall
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [0.0, 0.0, -1.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [0.0, 0.0, -1.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [0.0, 0.0, -1.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [0.0, 0.0, -1.0],
			color: [0.5, 0.5, 0.5],
		},
		// left wall
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, -1.0, -1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, 1.0, -1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		// right wall
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, -1.0, -1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, 1.0, -1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [-1.0, 0.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		// top wall
		Vertex {
			position: [-1.0, -1.0, -1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, -1.0, -1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, -1.0, 1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, -1.0, 1.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		// bottom wall
		Vertex {
			position: [-1.0, 1.0, -1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, 1.0, -1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [1.0, 1.0, 1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
		Vertex {
			position: [-1.0, 1.0, 1.0],
			normal: [0.0, -1.0, 0.0],
			color: [0.5, 0.5, 0.5],
		},
	];
}

impl crate::scene::Scene for BouncingCubeScene {
	fn resize(
		&mut self,
		device: &wgpu::Device,
		surface_configuration: &wgpu::SurfaceConfiguration,
	) {
		self.bouncing_cube_model.resize(
			surface_configuration.width as f32,
			surface_configuration.height as f32,
		);
		self.depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration,
			"Bouncing cube scene",
		);
	}

	fn update(&mut self, dt: f32) {
		self.bouncing_cube_model.update(dt);
	}

	fn render(
		&mut self,
		command_encoder: &mut wgpu::CommandEncoder,
		queue: &wgpu::Queue,
		output_texture_view: &wgpu::TextureView,
	) {
		// TODO: this only needs to be done once
		queue.write_buffer(
			&self.vertex_buffer,
			0,
			bytemuck::cast_slice(
				&Self::CUBE_VERTICES
					.into_iter()
					.chain(Self::WALL_COORDINATES.into_iter())
					.collect::<Vec<_>>(),
			),
		);

		// Write uniforms.
		queue.write_buffer(
			&self.render_camera_uniform_buffer,
			0,
			bytemuck::bytes_of(&self.bouncing_cube_model.scene_camera.transformation),
		);
		let instance_model_transforms =
			std::iter::repeat(glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(
					self.bouncing_cube_model.cube.size / 2.0, // defined cube vertices give cube a side length of 2, not 1
					self.bouncing_cube_model.cube.size / 2.0,
					self.bouncing_cube_model.cube.size / 2.0,
				),
				glam::Quat::from_axis_angle(
					self.bouncing_cube_model.cube.axis_of_rotation.into(),
					self.bouncing_cube_model.cube.rotation_angle,
				),
				self.bouncing_cube_model.cube.center.into(),
			))
			.take(6)
			.chain(
				std::iter::repeat(glam::Mat4::from_scale(glam::Vec3::from_slice(
					&self.bouncing_cube_model.scene_bounds,
				)))
				.take(5),
			)
			.collect::<Vec<_>>();
		let instance_model_dynamic_uniform_buffer_data = &instance_model_transforms
			.iter()
			.map(|matrix| InstanceTransform {
				matrix: matrix.to_cols_array_2d(),
			})
			.collect::<Vec<_>>();
		let instance_normal_dynamic_uniform_buffer_data = &instance_model_transforms
			.iter()
			.map(|matrix| InstanceTransform {
				matrix: glam::Mat4::from_mat3(
					glam::Mat3A::from_mat4(*matrix).inverse().transpose().into(),
				)
				.to_cols_array_2d(),
			})
			.collect::<Vec<_>>();
		queue.write_buffer(&self.instance_model_dynamic_uniform_buffer, 0, unsafe {
			std::slice::from_raw_parts(
				instance_model_dynamic_uniform_buffer_data.as_ptr() as *const u8,
				instance_model_dynamic_uniform_buffer_data.len()
					* self.instance_buffer_spacing as usize,
			)
		});
		queue.write_buffer(&self.instance_normal_dynamic_uniform_buffer, 0, unsafe {
			std::slice::from_raw_parts(
				instance_normal_dynamic_uniform_buffer_data.as_ptr() as *const u8,
				instance_normal_dynamic_uniform_buffer_data.len()
					* self.instance_buffer_spacing as usize,
			)
		});
		queue.write_buffer(
			&self.light_information_buffer,
			0,
			bytemuck::cast_slice(
				&self
					.bouncing_cube_model
					.lights
					.iter()
					.map(|light| LightInformationDatum {
						position: light.position.into(),
						_padding_0: 0,
						diffuse_color: light.diffuse_light.into(),
						_padding_1: 0,
					})
					.collect::<Vec<_>>(),
			),
		);

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
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_push_constants(
			wgpu::ShaderStages::FRAGMENT,
			0,
			bytemuck::bytes_of(&glam::Vec3::from(self.bouncing_cube_model.ambient_light)),
		);
		render_pass.set_bind_group(0, &self.render_camera_bind_group, &[]);
		render_pass.set_bind_group(2, &self.light_information_bind_group, &[]);
		for i in 0u32..11 {
			let offset =
				i as wgpu::DynamicOffset * self.instance_buffer_spacing as wgpu::DynamicOffset;
			render_pass.set_bind_group(1, &self.instance_bind_group, &[offset, offset]);
			render_pass.draw_indexed(6 * i..6 * i + 6, 0, 0..1);
		}
	}
}
