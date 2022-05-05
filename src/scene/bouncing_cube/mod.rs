mod bouncing_cube_model;
use wgpu::util::DeviceExt;

/**
 * TODO:
 *  * make a transformation struct in utilities and instance the vertices so that each face is an instance with its own transformation
 *		this is so that walls can be transformed separately from cube faces
 *  * point light cube shadows on wall
 *  * make shadow maps not change on screen size
 *  * blinn-phong lighting
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	normal: [f32; 3],
	color: [f32; 3],
}

pub struct BouncingCubeScene {
	bouncing_cube_model: bouncing_cube_model::BouncingCubeSceneInformation,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	render_camera_uniform_buffer: wgpu::Buffer,
	render_camera_bind_group: wgpu::BindGroup,
	depth_texture: crate::scene::utilities::texture::Texture,
}

impl BouncingCubeScene {
	pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
		let bouncing_cube_model = bouncing_cube_model::BouncingCubeSceneInformation::new(
			surface_configuration.width as f32,
			surface_configuration.height as f32,
		);
		let render_shader_module = device.create_shader_module(&wgpu::include_wgsl!("render.wgsl"));
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
		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Bouncing cube scene pipeline layout"),
				bind_group_layouts: &[&render_camera_bind_group_layout],
				push_constant_ranges: &[],
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
		self.bouncing_cube_model.resize(surface_configuration.width as f32, surface_configuration.height as f32);
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
		// TODO:
	}
}
