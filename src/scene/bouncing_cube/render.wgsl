struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec3<f32>;
};

struct Transform {
	transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera_transform: Transform;
[[group(1), binding(0)]]
var<uniform> object_transform: Transform;

struct FragmentInput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] world_position: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	return FragmentInput(
		camera_transform.transformation * object_transform.transformation * vec4<f32>(input.position, 1.0),
		vec4<f32>(input.position, 1.0),
		vec4<f32>(input.normal, 0.0),
		vec4<f32>(input.color, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
