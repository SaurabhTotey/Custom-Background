struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec3<f32>;
};

struct CubeTransform {
	camera_transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> cube_transform: CubeTransform;

struct FragmentInput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] world_position: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct LightInformation {
	position: vec3<f32>;
	ambient_light: vec3<f32>;
	diffuse_light: vec3<f32>;
	constant_attenuation_term: f32;
	linear_attenuation_term: f32;
	quadratic_attenuation_term: f32;
};
[[group(1), binding(0)]]
var<uniform> cube_light: LightInformation;

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	return FragmentInput(
		cube_transform.camera_transformation * vec4<f32>(input.position, 1.0),
		vec4<f32>(input.position, 1.0),
		vec4<f32>(input.normal, 0.0),
		vec4<f32>(input.color, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	let position_to_light_vector = cube_light.position - input.world_position.xyz;
	let light_distance = length(position_to_light_vector);
	let light_direction = normalize(position_to_light_vector);
	let diffuse_lighting = max(dot(light_direction, input.normal.xyz), 0.0) * cube_light.diffuse_light;
	let attenuation = 1.0 / (cube_light.constant_attenuation_term + cube_light.linear_attenuation_term * light_distance+ cube_light.quadratic_attenuation_term * light_distance * light_distance);
	let total_lighting = vec4<f32>(attenuation * min(diffuse_lighting + cube_light.ambient_light, vec3<f32>(1.0)), 1.0);
	return FragmentOutput(total_lighting * input.color);
}
