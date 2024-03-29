struct CameraUniform {
    view_proj: mat4x4<f32>
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
	@location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords: vec2<f32>,
	@location(1) rev_tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	let world_position_homogenous = vec4(input.position, 0.0, 1.0);
	// let position = camera.view_proj * world_position_homogenous;
	let position = world_position_homogenous;
	out.clip_position = position;
	out.tex_coords = input.tex_coords;
	out.rev_tex_coords = vec2<f32>(input.tex_coords.x, 1. - input.tex_coords.y);
    return out;
}

@group(0) @binding(0)
var sprite_texture: texture_2d<f32>;

@group(0) @binding(1)
var sdf_texture: texture_2d<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

const light: vec2<f32> = vec2<f32>(200., 200.);
const max_int: f32 = 1.;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	var occluder_distance = textureSample(sdf_texture, texture_sampler, in.rev_tex_coords);

	var distance_to_light = length(in.clip_position.xy - light);
	var light_intensity = max_int / (distance_to_light * distance_to_light);

	
	var sampled = textureSample(sprite_texture, texture_sampler, in.tex_coords);
	
	return sampled;
 //    return sampled;
}
