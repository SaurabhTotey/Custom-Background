struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec3<f32>;
};

struct CubeUniform {
	camera_transformation: mat4x4<f32>;
	world_transformation: mat4x4<f32>;
	ambient_light: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> cube_uniform: CubeUniform;

struct FragmentInput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	let new_position = cube_uniform.camera_transformation * cube_uniform.world_transformation * vec4<f32>(input.position, 1.0);
	let new_normal = cube_uniform.world_transformation * vec4<f32>(input.normal, 0.0);
	let new_color = vec4<f32>(cube_uniform.ambient_light * input.color, 1.0);
	return FragmentInput(
		new_position,
		new_color,
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
