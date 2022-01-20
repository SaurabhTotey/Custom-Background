struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec3<f32>;
};

struct CubeTransformation {
	camera_transformation: mat4x4<f32>;
	world_transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> transformations: CubeTransformation;

struct FragmentInput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	let new_position = transformations.camera_transformation * transformations.world_transformation * vec4<f32>(input.position.x, input.position.y, input.position.z, 1.0);
	let new_normal = transformations.world_transformation * vec4<f32>(input.normal.x, input.normal.y, input.normal.z, 0.0);
	return FragmentInput(
		new_position,
		vec4<f32>(input.color.x, input.color.y, input.color.z, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
