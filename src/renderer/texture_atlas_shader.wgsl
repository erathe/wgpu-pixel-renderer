struct TextureAtlasUniform {
	size: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>
};

@group(2) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
	@location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
	@location(2) size: vec2<f32>,
	@location(3) texture_origin: vec2<f32>,
	@location(4) translation: vec2<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords: vec2<f32>,
	@location(1) size: vec2<f32>,
	@location(2) texture_origin: vec2<f32>,
	@location(3) world_position: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput, ins: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	// TODO: should be a part of the instance buffer
	let scale_matrix = mat2x2<f32>(vec2<f32>(48.0, 0.0), vec2<f32>(0.0, 48.0));
	let world_position = ((input.position + 0.5) * scale_matrix) + ins.translation;
	let world_position_homogenous = vec4(world_position, 0.0, 1.0);
	let position = camera.view_proj * world_position_homogenous;
	out.clip_position = position;
	out.world_position = world_position;
	out.tex_coords = input.tex_coords;
	out.size = ins.size;
	out.texture_origin = ins.texture_origin;
    return out;
}


@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var sdf_texture: texture_2d<f32>;

@group(1) @binding(0)
var<uniform> atlas: TextureAtlasUniform;

//TODO: uniform
const screen = vec2(1920., 1200.);
const light = vec2(480., 550.);
const light_2 = vec2(1400., 550.);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	// sprite
	let sprite_size = in.size / atlas.size;
	let uvOffset = in.tex_coords * sprite_size;
	let localUv = uvOffset + (in.texture_origin / atlas.size);
	
	var sampled = textureSample(texture, texture_sampler, localUv);

	// lighting
	let world_uv = in.world_position / screen;

	let d = distance(in.world_position, light) - 100.;
	let color = 1. - step(0., d);

	let d_2 = distance(in.world_position, light_2) - 100.;
	let color_2 = 1. - step(0., d_2);
	


	let sdf_sample = textureSample(sdf_texture, texture_sampler, world_uv);
	return sampled + (sdf_sample / 200.) + (color * 0.1) + (color_2 * 0.1);
}
