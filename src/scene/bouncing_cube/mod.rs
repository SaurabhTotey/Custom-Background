mod bouncing_cube_model;
use wgpu::util::DeviceExt;

/**
 * TODO:
 *  * shadow mapping for point lights
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
	position: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
	shininess: f32,
	ambient_color: [f32; 3],
	diffuse_color: [f32; 3],
	specular_color: [f32; 3],
	object_transform: [[f32; 4]; 4],
	normal_transform: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightInformationDatum {
	position: [f32; 3],
	_padding_0: u32,
	ambient_color: [f32; 3],
	_padding_1: u32,
	diffuse_color: [f32; 3],
	_padding_2: u32,
	specular_color: [f32; 3],
	_padding_3: u32,
	constant_attenuation: f32,
	linear_attenuation: f32,
	quadratic_attenuation: f32,
	_padding_4: u32,
}

pub struct BouncingCubeScene {
	bouncing_cube_model: bouncing_cube_model::BouncingCubeSceneInformation,
	quad_transforms: [glam::Mat4; 11],
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	render_camera_uniform_buffer: wgpu::Buffer,
	render_camera_bind_group: wgpu::BindGroup,
	instance_buffer: wgpu::Buffer,
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

		// Make the quad transforms since all drawn quads are instanced from the same quad of unit sidelength centered in the xy plane.
		// The quad transform rotations are incredibly important because the normal must also be transformed correctly.
		let quad_transforms = [
			glam::Mat4::from_rotation_translation(
				glam::Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
				-0.5 * glam::Vec3::X,
			), // left cube quad
			glam::Mat4::from_rotation_translation(
				glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
				0.5 * glam::Vec3::X,
			), // right cube quad
			glam::Mat4::from_rotation_translation(
				glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
				0.5 * glam::Vec3::Y,
			), // top cube quad
			glam::Mat4::from_rotation_translation(
				glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
				-0.5 * glam::Vec3::Y,
			), // bottom cube quad
			glam::Mat4::from_translation(0.5 * glam::Vec3::Z), // back cube quad
			glam::Mat4::from_rotation_translation(
				glam::Quat::from_rotation_x(std::f32::consts::PI),
				-0.5 * glam::Vec3::Z,
			), // front cube quad
			glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(2.0, 2.0, 2.0),
				glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
				-glam::Vec3::X,
			), // left wall quad
			glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(2.0, 2.0, 2.0),
				glam::Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
				glam::Vec3::X,
			), // right wall quad
			glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(2.0, 2.0, 2.0),
				glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
				glam::Vec3::Y,
			), // top wall quad
			glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(2.0, 2.0, 2.0),
				glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
				-glam::Vec3::Y,
			), // bottom wall quad
			glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(2.0, 2.0, 2.0),
				glam::Quat::from_rotation_x(std::f32::consts::PI),
				glam::Vec3::Z,
			), // back wall quad
		];

		// Get shader.
		let render_shader_module = device.create_shader_module(&wgpu::include_wgsl!("render.wgsl"));

		// Create buffers and bind groups.
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene vertex buffer"),
			contents: bytemuck::cast_slice(&[
				QuadVertex {
					position: [-0.5, 0.5],
				},
				QuadVertex {
					position: [-0.5, -0.5],
				},
				QuadVertex {
					position: [0.5, -0.5],
				},
				QuadVertex {
					position: [0.5, 0.5],
				},
			]),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bouncing cube scene index buffer"),
			contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 2, 3]),
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
			surface_configuration.width,
			surface_configuration.height,
			"Bouncing cube scene",
		);

		// Create the instance buffer.
		let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Bouncing cube scene instance model dynamic uniform buffer"),
			size: 11 * std::mem::size_of::<InstanceData>() as wgpu::BufferAddress, // 11 quads: 6 for the cube and 5 for the walls
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false,
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
					&light_information_bind_group_layout,
				],
				push_constant_ranges: &[wgpu::PushConstantRange {
					stages: wgpu::ShaderStages::FRAGMENT,
					range: 0..12, // a vec3 is 12 bytes (for 3 floats of 4 bytes each)
				}],
			});
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Bouncing cube scene pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &render_shader_module,
				entry_point: "vertex_stage",
				buffers: &[
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &wgpu::vertex_attr_array![0 => Float32x2],
					},
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &wgpu::vertex_attr_array![1 => Float32, 2 => Float32x3, 3 => Float32x3, 4 => Float32x3, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Float32x4, 10 => Float32x4, 11 => Float32x4, 12 => Float32x4],
					}
				],
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
			quad_transforms,
			render_pipeline,
			vertex_buffer,
			index_buffer,
			render_camera_uniform_buffer,
			render_camera_bind_group,
			instance_buffer,
			light_information_buffer,
			light_information_bind_group,
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
		self.bouncing_cube_model.resize(
			surface_configuration.width as f32,
			surface_configuration.height as f32,
		);
		self.depth_texture = crate::scene::utilities::texture::Texture::create_depth_texture(
			device,
			surface_configuration.width,
			surface_configuration.height,
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
		// Write instance data.
		let instance_model_transforms =
			std::iter::repeat(glam::Mat4::from_scale_rotation_translation(
				glam::Vec3::new(
					self.bouncing_cube_model.cube.side_length,
					self.bouncing_cube_model.cube.side_length,
					self.bouncing_cube_model.cube.side_length,
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
			.zip(&self.quad_transforms)
			.map(|(world_transform, model_transform)| world_transform * *model_transform)
			.collect::<Vec<_>>();
		let instance_buffer_data = &instance_model_transforms
			.iter()
			.zip(
				(&self.bouncing_cube_model.cube.quads)
					.iter()
					.chain(&self.bouncing_cube_model.wall_quads),
			)
			.map(|(model_transform, quad_data)| InstanceData {
				shininess: quad_data.shininess,
				ambient_color: quad_data.ambient_color,
				diffuse_color: quad_data.diffuse_color,
				specular_color: quad_data.specular_color,
				object_transform: model_transform.to_cols_array_2d(),
				normal_transform: model_transform.inverse().transpose().to_cols_array_2d(),
			})
			.collect::<Vec<_>>();
		queue.write_buffer(
			&self.instance_buffer,
			0,
			bytemuck::cast_slice(&instance_buffer_data),
		);

		// Write uniforms.
		queue.write_buffer(
			&self.render_camera_uniform_buffer,
			0,
			bytemuck::bytes_of(&self.bouncing_cube_model.scene_camera.transformation),
		);
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
						ambient_color: light.ambient_light,
						diffuse_color: light.diffuse_light,
						specular_color: light.specular_light,
						constant_attenuation: light.constant_attenuation,
						linear_attenuation: light.linear_attenuation,
						quadratic_attenuation: light.quadratic_attenuation,
						_padding_0: 0,
						_padding_1: 0,
						_padding_2: 0,
						_padding_3: 0,
						_padding_4: 0,
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
		render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_push_constants(
			wgpu::ShaderStages::FRAGMENT,
			0,
			bytemuck::bytes_of(&glam::Vec3::from(
				self.bouncing_cube_model.scene_camera.position,
			)),
		);
		render_pass.set_bind_group(0, &self.render_camera_bind_group, &[]);
		render_pass.set_bind_group(1, &self.light_information_bind_group, &[]);
		render_pass.draw_indexed(0..6, 0, 0..11);
	}
}
