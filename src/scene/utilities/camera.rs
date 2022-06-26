pub struct Camera {
	pub field_of_view: f32,
	pub aspect_ratio: f32,
	pub near_plane_distance: f32,
	pub far_plane_distance: f32,
	pub position: glam::Vec3A,
	pub look_direction: glam::Vec3A,
	pub up_direction: glam::Vec3A,
	pub right_direction: glam::Vec3A,
	pub yaw: f32,
	pub pitch: f32,
	pub transformation: glam::Mat4,
	pub view_planes: [glam::Vec4; 6],
}
impl Camera {
	pub fn new(field_of_view: f32, aspect_ratio: f32) -> Self {
		let mut camera = Self {
			field_of_view,
			aspect_ratio,
			near_plane_distance: 0.001,
			far_plane_distance: 1000.0,
			position: glam::Vec3A::new(0.0, 0.0, -2.0),
			look_direction: glam::Vec3A::Z,
			up_direction: -glam::Vec3A::Y,
			right_direction: glam::Vec3A::X,
			yaw: std::f32::consts::PI / 2.0,
			pitch: 0f32,
			transformation: glam::Mat4::IDENTITY,
			view_planes: [glam::Vec4::ZERO; 6],
		};
		camera.recalculate_transformation_and_view_planes();
		camera
	}

	/**
	 * Mutate the camera by rotating its look and right directions by the given yaw and pitch.
	 * Take note that the up direction does not change.
	 * It is likely prudent to call recalculate_transformation_and_view_planes after calling this method.
	 */
	pub fn rotate(&mut self, yaw: f32, pitch: f32) {
		let mut new_pitch: f32 = self.pitch + pitch;
		if new_pitch <= -std::f32::consts::PI / 2.0 || std::f32::consts::PI / 2.0 <= new_pitch {
			new_pitch = self.pitch;
		}
		self.yaw += yaw;
		let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
		let (pitch_sin, pitch_cos) = new_pitch.sin_cos();
		self.look_direction =
			glam::Vec3A::new(-yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos);
		self.right_direction = self.look_direction.cross(self.up_direction);
		self.pitch = new_pitch;
	}

	/**
	 * Recalculate the transformation matrix and view planes for this camera and store them in the camera.
	 */
	pub fn recalculate_transformation_and_view_planes(&mut self) {
		// Calculate the new tranformation matrix.
		let view_matrix = glam::Mat4::look_at_rh(
			self.position.into(),
			(self.position + self.look_direction).into(),
			self.up_direction.into(),
		);
		let projection_matrix = glam::Mat4::perspective_rh(
			self.field_of_view,
			self.aspect_ratio,
			self.near_plane_distance,
			self.far_plane_distance,
		);
		self.transformation = projection_matrix * view_matrix;
		let transformation = self.transformation.to_cols_array_2d();

		// Calculate the view planes from the transformation and then normalize them.
		for face in 0..6 {
			for i in 0..4 {
				let mut rhs = transformation[i][face / 2];
				if face % 2 == 1 {
					rhs = -rhs;
				}
				self.view_planes[face][i] = transformation[i][3] + rhs;
			}
		}

		// Normalize view planes
		for i in 0..6 {
			self.view_planes[i] =
				self.view_planes[i] / glam::Vec3A::from(self.view_planes[i]).length();
		}
	}

	/**
	 * Calculate whether this Camera can see any point within the fudge_radius of the given point.
	 */
	pub fn can_see(&self, point: glam::Vec3A, fudge_radius: f32) -> bool {
		self.view_planes
			.iter()
			.all(|&plane| point.dot(glam::Vec3A::from(plane)) + plane.w + fudge_radius >= 0.0)
	}

	/**
	 * Creates a bind group that only binds a buffer meant to store this camera.
	 */
	pub fn create_bind_group(
		&self,
		device: &wgpu::Device,
		label: &str,
	) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
		let label = label.to_owned() + " camera";
		let transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some(&(label.clone() + " uniform buffer")),
			size: std::mem::size_of::<glam::Mat4>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		let transform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some(&(label.clone() + " bind group layout")),
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
		let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some(&(label.clone() + " bind group")),
			layout: &transform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: transform_uniform_buffer.as_entire_binding(),
			}],
		});
		(
			transform_uniform_buffer,
			transform_bind_group_layout,
			transform_bind_group,
		)
	}
}
