use rand::Rng;

pub struct BouncingCubeSceneInformation {
	pub window_size: [f32; 2],
	pub scene_camera: crate::scene::utilities::camera::Camera,
	pub scene_bounds: [f32; 3],
	pub ambient_light: glam::Vec3A,
	pub cube: CubeInformation,
	pub lights: [PointLightInformation; 3],
}

pub struct PointLightInformation {
	pub position: glam::Vec3A,
	pub diffuse_light: glam::Vec3A,
}

pub struct CubeInformation {
	pub center: glam::Vec3A,
	pub size: f32,
	pub velocity: glam::Vec3A,
	pub rotation_angle: f32,
	pub axis_of_rotation: glam::Vec3A,
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
		let size = 0.1;
		let cube_semi_diagonal_length = f32::sqrt(3.0 * (size / 2.0) * (size / 2.0));
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
			size,
			velocity: rng.gen::<glam::Vec3A>().normalize() * 1.5,
			rotation_angle: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
			axis_of_rotation: rng.gen::<glam::Vec3A>().normalize(),
		};
		Self {
			window_size: [width, height],
			scene_camera,
			scene_bounds: [x_bound, y_bound, z_bound],
			ambient_light: glam::Vec3A::X,
			cube,
			lights: [
				// TODO: better way of setting the lights' initial positions
				PointLightInformation {
					position: glam::Vec3A::new(0.0, 0.5, -2.1),
					diffuse_light: glam::Vec3A::new(1.0, 0.0, 0.0),
				},
				PointLightInformation {
					position: glam::Vec3A::new(-0.5, 0.0, -2.1),
					diffuse_light: glam::Vec3A::new(0.0, 1.0, 0.0),
				},
				PointLightInformation {
					position: glam::Vec3A::new(0.5, 0.0, -2.1),
					diffuse_light: glam::Vec3A::new(0.0, 0.0, 1.0),
				},
			],
		}
	}

	pub fn resize(&mut self, width: f32, height: f32) {
		self.scene_camera.aspect_ratio = width / height;
		self.scene_camera
			.recalculate_transformation_and_view_planes();
		self.scene_bounds[0] = self.scene_bounds[1] * self.scene_camera.aspect_ratio;
		// TODO: here, it is possible for the cube to go out of bounds because the bounds change
	}

	pub fn update(&mut self, dt: f32) {
		self.cube.rotation_angle += std::f32::consts::FRAC_PI_4 * dt;
		self.cube.center += self.cube.velocity * dt;
		let light_rotation_matrix = glam::Mat4::from_rotation_z(std::f32::consts::FRAC_PI_4 * dt);
		self.lights.iter_mut().for_each(|light| { light.position = glam::Vec3A::from(light_rotation_matrix * glam::Vec4::from((light.position, 1.0))); });
		// TODO: do the math/physics and have more realistic collisions that affect the rotation -- the walls can only apply forces along their own normal on the touching/violating corners of the cube
		let cube_semi_diagonal_length =
			f32::sqrt(3.0 * (self.cube.size / 2.0) * (self.cube.size / 2.0));
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
