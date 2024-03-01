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
}

@vertex
fn vs_main(input: VertexInput, ins: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	let world_position = input.position + ins.translation;
	let world_position_homogenous = vec4(world_position, 0.0, 1.0);
	let position = camera.view_proj * world_position_homogenous;
	out.clip_position = position;
	out.tex_coords = input.tex_coords;
	out.size = ins.size;
	out.texture_origin = ins.texture_origin;
    return out;
}


@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> atlas: TextureAtlasUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let sprite_size = in.size / atlas.size;
	let uvOffset = in.tex_coords * sprite_size;

	let localUv = uvOffset + (in.texture_origin / atlas.size);
	
    return textureSample(texture, texture_sampler, localUv);
}

