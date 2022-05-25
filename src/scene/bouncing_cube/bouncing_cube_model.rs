use rand::Rng;

pub mod direction {
	pub const LEFT: u8 = 0;
	pub const RIGHT: u8 = 1;
	pub const TOP: u8 = 2;
	pub const BOTTOM: u8 = 3;
	pub const BACK: u8 = 4;
	pub const FRONT: u8 = 5;
}

pub struct BouncingCubeSceneInformation {
	pub window_size: [f32; 2],
	pub scene_camera: crate::scene::utilities::camera::Camera,
	pub scene_bounds: [f32; 3],
	pub ambient_light: glam::Vec3A,
	pub cube: CubeInformation,
	pub point_light_distance_from_center: f32,
	pub point_light_rotation_angle: f32,
	pub lights: [PointLightInformation; 3],
	pub wall_quads: [QuadInformation; 5],
}

pub struct PointLightInformation {
	pub position: glam::Vec3A,
	pub diffuse_light: glam::Vec3A,
}

pub struct CubeInformation {
	pub center: glam::Vec3A,
	pub side_length: f32,
	pub velocity: glam::Vec3A,
	pub rotation_angle: f32,
	pub axis_of_rotation: glam::Vec3A,
	pub quads: [QuadInformation; 6],
}

pub struct QuadInformation {
	pub color: [f32; 3],
}

impl BouncingCubeSceneInformation {
	pub fn new(width: f32, height: f32) -> Self {
		let field_of_view = std::f32::consts::PI / 2.0;
		let aspect_ratio = width / height;
		let scene_camera =
			crate::scene::utilities::camera::Camera::new(field_of_view, aspect_ratio);
		let y_bound = (field_of_view / 2.0).tan() * (-1.0 - scene_camera.position.z);
		let x_bound = y_bound * aspect_ratio;
		let z_bound = 1.0;
		let side_length = 0.1;
		let cube_semi_diagonal_length = f32::sqrt(3.0 * (side_length / 2.0) * (side_length / 2.0));
		let mut rng = rand::thread_rng();
		let cube = CubeInformation {
			center: glam::Vec3A::new(
				rng.gen_range(
					-x_bound + cube_semi_diagonal_length..x_bound - cube_semi_diagonal_length,
				),
				rng.gen_range(
					-y_bound + cube_semi_diagonal_length..y_bound - cube_semi_diagonal_length,
				),
				rng.gen_range(
					-z_bound + cube_semi_diagonal_length..z_bound - cube_semi_diagonal_length,
				),
			),
			side_length,
			velocity: rng.gen::<glam::Vec3A>().normalize() * 1.5,
			rotation_angle: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
			axis_of_rotation: rng.gen::<glam::Vec3A>().normalize(),
			quads: [
				QuadInformation {
					color: [1.0, 0.1, 0.1],
				},
				QuadInformation {
					color: [1.0, 0.1, 0.1],
				},
				QuadInformation {
					color: [0.1, 1.0, 0.1],
				},
				QuadInformation {
					color: [0.1, 1.0, 0.1],
				},
				QuadInformation {
					color: [0.1, 0.1, 1.0],
				},
				QuadInformation {
					color: [0.1, 0.1, 1.0],
				},
			],
		};
		let point_light_distance_from_center = y_bound.min(x_bound) / 3.0;
		Self {
			window_size: [width, height],
			scene_camera,
			scene_bounds: [x_bound, y_bound, z_bound],
			ambient_light: glam::Vec3A::new(0.1, 0.1, 0.1),
			cube,
			point_light_distance_from_center,
			point_light_rotation_angle: 0.0,
			lights: [
				PointLightInformation {
					position: glam::Vec3A::new(
						point_light_distance_from_center * 0f32.cos(),
						point_light_distance_from_center * 0f32.sin(),
						-2.1,
					),
					diffuse_light: glam::Vec3A::new(1.0, 0.1, 0.1),
				},
				PointLightInformation {
					position: glam::Vec3A::new(
						point_light_distance_from_center
							* (2.0 * std::f32::consts::FRAC_PI_3).cos(),
						point_light_distance_from_center
							* (2.0 * std::f32::consts::FRAC_PI_3).sin(),
						-2.1,
					),
					diffuse_light: glam::Vec3A::new(0.1, 1.0, 0.1),
				},
				PointLightInformation {
					position: glam::Vec3A::new(
						point_light_distance_from_center
							* (4.0 * std::f32::consts::FRAC_PI_3).cos(),
						point_light_distance_from_center
							* (4.0 * std::f32::consts::FRAC_PI_3).sin(),
						-2.1,
					),
					diffuse_light: glam::Vec3A::new(0.1, 0.1, 1.0),
				},
			],
			wall_quads: [
				QuadInformation { color: [0.5; 3] },
				QuadInformation { color: [0.5; 3] },
				QuadInformation { color: [0.5; 3] },
				QuadInformation { color: [0.5; 3] },
				QuadInformation { color: [0.5; 3] },
			],
		}
	}

	pub fn resize(&mut self, width: f32, height: f32) {
		self.scene_camera.aspect_ratio = width / height;
		self.scene_camera
			.recalculate_transformation_and_view_planes();
		self.scene_bounds[0] = self.scene_bounds[1] * self.scene_camera.aspect_ratio;
		self.point_light_distance_from_center =
			self.scene_bounds[1].min(self.scene_bounds[0]) / 3.0;
		// TODO: here, it is possible for the cube to go out of bounds because the bounds change
	}

	pub fn update(&mut self, dt: f32) {
		self.cube.rotation_angle += std::f32::consts::FRAC_PI_4 * dt;
		self.point_light_rotation_angle += std::f32::consts::FRAC_PI_2 * dt;
		self.cube.center += self.cube.velocity * dt;
		(0..self.lights.len()).for_each(|light_index| {
			let relative_angle = 2.0 * light_index as f32 * std::f32::consts::FRAC_PI_3;
			let absolute_angle = self.point_light_rotation_angle + relative_angle;
			self.lights[light_index].position.x =
				self.point_light_distance_from_center * absolute_angle.cos();
			self.lights[light_index].position.y =
				self.point_light_distance_from_center * absolute_angle.sin();
		});
		// TODO: do the math/physics and have more realistic collisions that affect the rotation -- the walls can only apply forces along their own normal on the touching/violating corners of the cube
		let cube_semi_diagonal_length =
			f32::sqrt(3.0 * (self.cube.side_length / 2.0) * (self.cube.side_length / 2.0));
		if self.cube.center.x - cube_semi_diagonal_length <= -self.scene_bounds[0] {
			self.cube.center.x = -self.scene_bounds[0] + cube_semi_diagonal_length;
			self.cube.velocity.x *= -1.0;
		}
		if self.cube.center.x + cube_semi_diagonal_length >= self.scene_bounds[0] {
			self.cube.center.x = self.scene_bounds[0] - cube_semi_diagonal_length;
			self.cube.velocity.x *= -1.0;
		}
		if self.cube.center.y - cube_semi_diagonal_length <= -self.scene_bounds[1] {
			self.cube.center.y = -self.scene_bounds[1] + cube_semi_diagonal_length;
			self.cube.velocity.y *= -1.0;
		}
		if self.cube.center.y + cube_semi_diagonal_length >= self.scene_bounds[1] {
			self.cube.center.y = self.scene_bounds[1] - cube_semi_diagonal_length;
			self.cube.velocity.y *= -1.0;
		}
		if self.cube.center.z - cube_semi_diagonal_length <= -self.scene_bounds[2] {
			self.cube.center.z = -self.scene_bounds[2] + cube_semi_diagonal_length;
			self.cube.velocity.z *= -1.0;
		}
		if self.cube.center.z + cube_semi_diagonal_length >= self.scene_bounds[2] {
			self.cube.center.z = self.scene_bounds[2] - cube_semi_diagonal_length;
			self.cube.velocity.z *= -1.0;
		}
	}
}
