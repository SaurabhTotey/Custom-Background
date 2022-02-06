pub struct SceneInformation {
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
	pub cube_center: glam::Vec3A,
	pub cube_size: f32,
	pub cube_velocity: glam::Vec3A,
	pub rotation_angle: f32,
	pub axis_of_rotation: glam::Vec3A,
}
