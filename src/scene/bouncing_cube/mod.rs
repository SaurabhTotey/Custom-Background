use rand::Rng;
use wgpu::util::DeviceExt;

/**
 * TODO:
 *  * diffuse lighting
 *  * have walls that represent where the cube is colliding and have lighting effect them too
 *  * blinn-phong shading
 *  * experiment with multiple lights
 *  * texture cube with die faces
 *  * make light move (maybe with time of day?)
 *  * consider more accurate collisions (probably won't do this)
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BouncingCubeVertex {
	position: [f32; 3],
	normal: [f32; 3],
	color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BouncingCubeTransformationUniform {
	camera_transformation: [[f32; 4]; 4],
	world_transformation: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BouncingCubeLightingUniform {
	position: [f32; 4], // only first three components are used; fourth component for padding
	ambient_light: [f32; 4], // only first three components are used; fourth component for padding
	diffuse_light: [f32; 4], // only first three components are used; fourth component for padding
	constant_attenuation_term: f32,
	linear_attenuation_term: f32,
	quadratic_attenuation_term: f32,
	_padding: f32,
}

pub struct BouncingCubeScene {
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	cube_transform_uniform_buffer: wgpu::Buffer,
	cube_transform_uniform_bind_group: wgpu::BindGroup,
	cube_light: BouncingCubeLightingUniform,
	cube_light_uniform_buffer: wgpu::Buffer,
	cube_light_uniform_bind_group: wgpu::BindGroup,
	depth_texture: crate::scene::utilities::texture::Texture,
	camera: crate::scene::utilities::camera::Camera,
	cube_position: glam::Vec3A,
	cube_velocity: glam::Vec3A,
	cube_rotation_angle: f32,
	cube_rotation_axis: glam::Vec3A,
	x_bound: f32,
	y_bound: f32,
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
					normal: [0.0, 0.0, -1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
					normal: [0.0, 0.0, -1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
					normal: [0.0, 0.0, -1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
					normal: [0.0, 0.0, -1.0],
					color: [1.0, 0.0, 0.0],
				},
				// front face
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
					normal: [0.0, 0.0, 1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
					normal: [0.0, 0.0, 1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
					normal: [0.0, 0.0, 1.0],
					color: [1.0, 0.0, 0.0],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
					normal: [0.0, 0.0, 1.0],
					color: [1.0, 0.0, 0.0],
				},
				// left face
				BouncingCubeVertex {
					position: [-0.1, -0.1, -0.1],
					normal: [-1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
					normal: [-1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
					normal: [-1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
					normal: [-1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				// right face
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
					normal: [1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
					normal: [1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
					normal: [1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
					normal: [1.0, 0.0, 0.0],
					color: [0.0, 1.0, 0.0],
				},
				// bottom face
				BouncingCubeVertex {
					position: [-0.1, -0.1, 0.1],
					normal: [0.0, -1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, 0.1],
					normal: [0.0, -1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [0.1, -0.1, -0.1],
					normal: [0.0, -1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [-0.1, -0.1, -0.1],
					normal: [0.0, -1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				// top face
				BouncingCubeVertex {
					position: [-0.1, 0.1, 0.1],
					normal: [0.0, 1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, 0.1],
					normal: [0.0, 1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [0.1, 0.1, -0.1],
					normal: [0.0, 1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
				BouncingCubeVertex {
					position: [-0.1, 0.1, -0.1],
					normal: [0.0, 1.0, 0.0],
					color: [0.0, 0.0, 1.0],
				},
			]),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene index buffer"),
			contents: bytemuck::cast_slice(
				&(0..6)
					.flat_map(|face| [0u16, 1, 2, 0, 2, 3].iter().map(move |i| face * 4 + i))
					.collect::<Vec<_>>(),
			),
			usage: wgpu::BufferUsages::INDEX,
		});
		let cube_transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene cube transform uniform buffer"),
			size: std::mem::size_of::<BouncingCubeTransformationUniform>() as wgpu::BufferAddress,
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
		let cube_light_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene cube light uniform buffer"),
			size: std::mem::size_of::<BouncingCubeLightingUniform>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		let cube_light_uniform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Bouncing cube scene cube light bind group layout"),
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
		let cube_light_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Bouncing cube scene cube light bind group"),
			layout: &cube_light_uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: cube_light_uniform_buffer.as_entire_binding(),
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
				bind_group_layouts: &[
					&cube_transform_uniform_bind_group_layout,
					&cube_light_uniform_bind_group_layout,
				],
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
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3],
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
		let camera = crate::scene::utilities::camera::Camera::new(
			std::f32::consts::PI / 2.0,
			surface_configuration.width as f32 / surface_configuration.height as f32,
		);
		let mut rng = rand::thread_rng();
		let y_bound = (camera.field_of_view / 2.0).tan() * (-1.0 - camera.position.z);
		let x_bound = y_bound * camera.aspect_ratio;
		let cube_position = glam::Vec3A::new(
			rng.gen_range(-x_bound + 0.1..x_bound - 0.1),
			rng.gen_range(-y_bound + 0.1..y_bound - 0.1),
			rng.gen_range(-0.9..0.9),
		);
		let cube_velocity = rng.gen::<glam::Vec3A>().normalize() * 1.5;
		let cube_rotation_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
		let cube_rotation_axis = rng.gen::<glam::Vec3A>().normalize();
		let cube_light = BouncingCubeLightingUniform {
			position: [x_bound, y_bound, -1.0, 1.0],
			ambient_light: [0.1; 4],
			diffuse_light: [0.0; 4],
			constant_attenuation_term: 0.0,
			linear_attenuation_term: 0.0,
			quadratic_attenuation_term: 0.0,
			_padding: 0.0,
		};
		Self {
			render_pipeline,
			vertex_buffer,
			index_buffer,
			cube_transform_uniform_buffer,
			cube_transform_uniform_bind_group,
			cube_light,
			cube_light_uniform_buffer,
			cube_light_uniform_bind_group,
			depth_texture,
			camera,
			cube_position,
			cube_velocity,
			cube_rotation_angle,
			cube_rotation_axis,
			x_bound,
			y_bound,
		}
	}
}

impl crate::scene::Scene for BouncingCubeScene {
	fn resize(
		&mut self,
		device: &wgpu::Device,
		surface_configuration: &wgpu::SurfaceConfiguration,
	) {
		self.camera.aspect_ratio =
			surface_configuration.width as f32 / surface_configuration.height as f32;
		self.camera.recalculate_transformation_and_view_planes();
		self.x_bound = self.y_bound * self.camera.aspect_ratio;
		self.cube_light.position[0] = self.x_bound;
		self.depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration,
			"Bouncing cube scene",
		);
	}

	fn update(&mut self, dt: f32) {
		self.cube_rotation_angle += std::f32::consts::FRAC_PI_4 * dt;
		self.cube_position += self.cube_velocity * dt;
		if self.cube_position.x < -self.x_bound + 0.1 {
			self.cube_position.x = -self.x_bound + 0.1;
			self.cube_velocity.x *= -1.0;
		}
		if self.cube_position.x > self.x_bound - 0.1 {
			self.cube_position.x = self.x_bound - 0.1;
			self.cube_velocity.x *= -1.0;
		}
		if self.cube_position.y < -self.y_bound + 0.1 {
			self.cube_position.y = -self.y_bound + 0.1;
			self.cube_velocity.y *= -1.0;
		}
		if self.cube_position.y > self.y_bound - 0.1 {
			self.cube_position.y = self.y_bound - 0.1;
			self.cube_velocity.y *= -1.0;
		}
		if self.cube_position.z < -0.9 {
			self.cube_position.z = -0.9;
			self.cube_velocity.z *= -1.0;
		}
		if self.cube_position.z > 0.9 {
			self.cube_position.z = 0.9;
			self.cube_velocity.z *= -1.0;
		}
	}

	fn render(
		&mut self,
		command_encoder: &mut wgpu::CommandEncoder,
		queue: &wgpu::Queue,
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
		let bouncing_cube_uniform = BouncingCubeTransformationUniform {
			camera_transformation: self.camera.transformation.to_cols_array_2d(),
			world_transformation: glam::Mat4::from_rotation_translation(
				glam::Quat::from_axis_angle(
					self.cube_rotation_axis.into(),
					self.cube_rotation_angle,
				),
				self.cube_position.into(),
			)
			.to_cols_array_2d(),
		};
		queue.write_buffer(
			&self.cube_transform_uniform_buffer,
			0,
			bytemuck::bytes_of(&bouncing_cube_uniform),
		);
		queue.write_buffer(
			&self.cube_light_uniform_buffer,
			0,
			bytemuck::bytes_of(&self.cube_light),
		);
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_bind_group(0, &self.cube_transform_uniform_bind_group, &[]);
		render_pass.set_bind_group(1, &self.cube_light_uniform_bind_group, &[]);
		render_pass.draw_indexed(0..36, 0, 0..1);
	}
}
