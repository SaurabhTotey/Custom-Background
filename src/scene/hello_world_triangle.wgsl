struct VertexInput {
	[[builtin(vertex_index)]] vertex_index: u32;
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
	var positions: array<vec4<f32>, 3u> = array<vec4<f32>, 3u>(
		vec4<f32>(-0.5, -0.5, 0.0, 1.0),
		vec4<f32>(0.5, -0.5, 0.0, 1.0),
		vec4<f32>(0.0, 0.5, 0.0, 1.0),
	);
	return FragmentInput(positions[input.vertex_index], vec4<f32>(1.0, 1.0, 1.0, 1.0));
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(input.color);
}
