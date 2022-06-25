struct VertexInput {
	@location(0) position: vec2<f32>,
};
struct InstanceInput {
	@location(1) shininess: f32,
	@location(2) ambient_color: vec3<f32>,
	@location(3) diffuse_color: vec3<f32>,
	@location(4) specular_color: vec3<f32>,
	@location(5) object_transform_col_0: vec4<f32>,
	@location(6) object_transform_col_1: vec4<f32>,
	@location(7) object_transform_col_2: vec4<f32>,
	@location(8) object_transform_col_3: vec4<f32>,
	@location(9) normal_transform_col_0: vec4<f32>,
	@location(10) normal_transform_col_1: vec4<f32>,
	@location(11) normal_transform_col_2: vec4<f32>,
	@location(12) normal_transform_col_3: vec4<f32>,
};

struct Transform {
	transformation: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera_transform: Transform;

struct FragmentInput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) world_position: vec4<f32>,
	@location(1) normal: vec4<f32>,
	@location(2) shininess: f32,
	@location(3) ambient_color: vec3<f32>,
	@location(4) diffuse_color: vec3<f32>,
	@location(5) specular_color: vec3<f32>,
};

struct LightInformationDatum {
	world_position: vec3<f32>,
	ambient_color: vec3<f32>,
	diffuse_color: vec3<f32>,
	specular_color: vec3<f32>,
	constant_attenuation: f32,
	linear_attenuation: f32,
	quadratic_attenuation: f32,
	camera_transformations: array<mat4x4<f32>, 6>,
};
struct PushConstantData {
	camera_position: vec3<f32>,
};

var<push_constant> push_constant_data: PushConstantData;
@group(1) @binding(0)
var<uniform> light_information: array<LightInformationDatum, 3>;

@group(2) @binding(0)
var total_shadow_map_textures: texture_depth_2d_array;
@group(2) @binding(1)
var total_shadow_map_sampler: sampler_comparison;

struct FragmentOutput {
	@location(0) color: vec4<f32>,
};

@vertex
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
		instance.shininess,
		instance.ambient_color,
		instance.diffuse_color,
		instance.specular_color,
	);
}

fn calculate_light_contribution(light_index: i32, fragment: FragmentInput) -> vec3<f32> {
	var light = light_information[light_index];
	let distance_to_light = length(light.world_position - fragment.world_position.xyz);
	let light_direction = normalize(light.world_position - fragment.world_position.xyz);
	let view_direction = normalize(push_constant_data.camera_position - fragment.world_position.xyz);
	let half_direction = normalize(view_direction + light_direction);
	let specular_amount = pow(max(0.0, dot(fragment.normal.xyz, half_direction)), 128.0 * fragment.shininess);
	let diffuse_amount = max(0.0, dot(fragment.normal.xyz, light_direction));
	let attenuation = 1.0 / (light.constant_attenuation + distance_to_light * light.linear_attenuation + distance_to_light * distance_to_light * light.quadratic_attenuation);

	var shadow_multiplier = 1.0;
	for (var i = 0; i < 6; i = i + 1) {
		let clip_position_according_to_light = light.camera_transformations[i] * fragment.world_position;
		if clip_position_according_to_light.w <= 0.0 {
			continue;
		}
		let projection_correction = 1.0 / clip_position_according_to_light.w;
		let projection_position = clip_position_according_to_light.xy * vec2<f32>(0.5, -0.5) * projection_correction + vec2<f32>(0.5, 0.5);
		shadow_multiplier *= textureSampleCompare(total_shadow_map_textures, total_shadow_map_sampler, projection_position, 6 * light_index + i, clip_position_according_to_light.z * projection_correction);
	}
	return attenuation * (light.ambient_color * fragment.ambient_color + shadow_multiplier * (diffuse_amount * light.diffuse_color * fragment.diffuse_color + specular_amount * light.specular_color * fragment.specular_color));
}

@fragment
fn fragment_stage(input: FragmentInput) -> FragmentOutput {
	var color = vec3<f32>(0.0);
	for (var i = 0; i < 3; i = i + 1) {
		color = color + calculate_light_contribution(i, input);
	}
	return FragmentOutput(vec4<f32>(color, 1.0));
}
