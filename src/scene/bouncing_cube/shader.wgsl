struct VertexInput {
	[[location(0)]] position: vec3<f32>;
};

struct CubeTransformation {
	transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> transformation: CubeTransformation;

struct FragmentInput {
	[[builtin(position)]] position: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	return FragmentInput(
		transformation.transformation * vec4<f32>(input.position.x, input.position.y, input.position.z, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(vec4<f32>(1.0, 1.0, 1.0, 1.0));
}
