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
struct SceneLightInformation {
	ambient_light: vec3<f32>;
	constant_attenuation: f32;
	linear_attenuation: f32;
	quadratic_attenuation: f32;
};

var<push_constant> scene_light_information: SceneLightInformation;
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

fn calculate_light_contribution(light: LightInformationDatum, normal: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
	let distance_to_light = length(light.world_position - world_position);
	let light_direction = normalize(light.world_position - world_position);
	let diffuse_amount = max(0.0, dot(normal, light_direction));
	let attenuation = 1.0 / (scene_light_information.constant_attenuation + distance_to_light * scene_light_information.linear_attenuation + distance_to_light * distance_to_light * scene_light_information.quadratic_attenuation);
	return attenuation * (scene_light_information.ambient_light + diffuse_amount * light.color);
}

fn approx_equals(a: f32, b: f32) -> bool {
	return a - 0.01 < b && a + 0.01 > b;
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	var color = vec3<f32>(0.0);
	for (var i = 0; i < 3; i = i + 1) {
		color = color + input.color.rgb * calculate_light_contribution(light_information.i[i], input.normal.xyz, input.world_position.xyz);
		// TODO: Temporary code to visualize light positions to figure out why one light is brighter than the rest on the back wall.
		if (approx_equals(input.world_position.x, light_information.i[i].world_position.x) && approx_equals(input.world_position.y, light_information.i[i].world_position.y)) {
			return FragmentOutput(vec4<f32>(light_information.i[i].color, 1.0));
		}
	}
	return FragmentOutput(vec4<f32>(color, 1.0));
}
