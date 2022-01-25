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

struct VertexOutput {
	[[builtin(position)]] clip_position: vec4<f32>;
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

[[stage(vertex)]]
fn vertex_stage(input: VertexInput) -> VertexOutput {
	return VertexOutput(
		cube_transform.camera_transformation * vec4<f32>(input.position, 1.0),
	);
}
