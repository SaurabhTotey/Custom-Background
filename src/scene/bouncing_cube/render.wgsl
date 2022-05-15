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
[[group(1), binding(1)]]
var<uniform> normal_transform: Transform;

struct FragmentInput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] world_position: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct LightInformationDatum {
	world_position: vec3<f32>;
	color: vec3<f32>;
};
// Well, this is annoying: I can't have the uniform be an array type, so I need it to be this wrapper type that has the array.
struct LightInformation {
	i: array<LightInformationDatum, 3>;
};

var<push_constant> ambient_light: vec3<f32>;
[[group(2), binding(0)]]
var<uniform> light_information: LightInformation;

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> FragmentInput {
	let world_position = object_transform.transformation * vec4<f32>(input.position, 1.0);
	return FragmentInput(
		camera_transform.transformation * world_position,
		world_position,
		normal_transform.transformation * vec4<f32>(input.normal, 1.0),
		vec4<f32>(input.color, 1.0),
	);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	return FragmentOutput(vec4<f32>(ambient_light, 1.0) * input.color);
}
