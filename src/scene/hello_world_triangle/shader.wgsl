struct VertexInput {
	[[location(0)]] position: vec2<f32>;
	[[location(1)]] color: vec3<f32>;
};

struct FragmentInput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	return FragmentInput(
		vec4<f32>(input.position.x, input.position.y, 0.0, 1.0),
		vec4<f32>(input.color.x, input.color.y, input.color.z, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
