struct VertexInput {
	[[location(0)]] position: vec2<f32>;
};
struct InstanceInput {
	[[location(1)]] color: vec3<f32>;
	[[location(2)]] object_transform_col_0: vec4<f32>;
	[[location(3)]] object_transform_col_1: vec4<f32>;
	[[location(4)]] object_transform_col_2: vec4<f32>;
	[[location(5)]] object_transform_col_3: vec4<f32>;
	[[location(6)]] normal_transform_col_0: vec4<f32>;
	[[location(7)]] normal_transform_col_1: vec4<f32>;
	[[location(8)]] normal_transform_col_2: vec4<f32>;
	[[location(9)]] normal_transform_col_3: vec4<f32>;
};

struct Transform {
	transformation: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera_transform: Transform;

struct FragmentInput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] world_position: vec4<f32>;
	[[location(1)]] normal: vec4<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct LightInformationDatum {
	world_position: vec3<f32>;
	color: vec3<f32>;
	constant_attenuation: f32;
	linear_attenuation: f32;
	quadratic_attenuation: f32;
};
// Well, this is annoying: I can't have the uniform be an array type, so I need it to be this wrapper type that has the array.
struct LightInformation {
	i: array<LightInformationDatum, 3>;
};
struct PushConstantData {
	ambient_light: vec3<f32>;
	camera_position: vec3<f32>;
};

var<push_constant> push_constant_data: PushConstantData;
[[group(1), binding(0)]]
var<uniform> light_information: LightInformation;

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_stage(vertex: VertexInput, instance: InstanceInput) -> FragmentInput {
	let object_transform = mat4x4<f32>(
		instance.object_transform_col_0,
		instance.object_transform_col_1,
		instance.object_transform_col_2,
		instance.object_transform_col_3,
	);
	let normal_transform = mat4x4<f32>(
		instance.normal_transform_col_0,
		instance.normal_transform_col_1,
		instance.normal_transform_col_2,
		instance.normal_transform_col_3,
	);
	let world_position = object_transform * vec4<f32>(vertex.position, 0.0, 1.0);
	return FragmentInput(
		camera_transform.transformation * world_position,
		world_position,
		normalize(normal_transform * vec4<f32>(0.0, 0.0, 1.0, 0.0)),
		vec4<f32>(instance.color, 1.0),
	);
}

fn calculate_light_contribution(light: LightInformationDatum, normal: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
	let distance_to_light = length(light.world_position - world_position);
	let light_direction = normalize(light.world_position - world_position);
	let diffuse_amount = max(0.0, dot(normal, light_direction));
	let attenuation = 1.0 / (light.constant_attenuation + distance_to_light * light.linear_attenuation + distance_to_light * distance_to_light * light.quadratic_attenuation);
	return attenuation * (push_constant_data.ambient_light + diffuse_amount * light.color);
}

[[stage(fragment)]]
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	var color = vec3<f32>(0.0);
	for (var i = 0; i < 3; i = i + 1) {
		color = color + input.color.rgb * calculate_light_contribution(light_information.i[i], input.normal.xyz, input.world_position.xyz);
	}
	return FragmentOutput(vec4<f32>(color, 1.0));
}
