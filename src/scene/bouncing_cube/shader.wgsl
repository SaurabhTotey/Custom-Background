struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec3<f32>;
};

struct CubeTransform {
	camera_transformation: mat4x4<f32>;
	world_transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> cube_transform: CubeTransform;

struct FragmentInput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] normal: vec4<f32>;
	[[location(1)]] color: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	return FragmentInput(
		cube_transform.camera_transformation * cube_transform.world_transformation * vec4<f32>(input.position, 1.0),
		cube_transform.world_transformation * vec4<f32>(input.normal, 0.0),
		vec4<f32>(input.color, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
