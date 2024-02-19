struct AtlasUniform {
	texture_atlas_size: vec2<u32>,
	tile_index: u32,
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
	@location(2) p: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords: vec2<f32>,
	@location(1) test: vec2<f32>
}

@vertex
fn vs_main(input: VertexInput, ins: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	let world_position = input.position + ins.p;
	let world_position_homogenous = vec4(world_position, 0.0, 1.0);
	let position = camera.view_proj * world_position_homogenous;
	out.clip_position = position;
	out.tex_coords = input.tex_coords;
    return out;
}


@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> atlas: AtlasUniform;

struct Animation {
	tile_index: u32,
}

@group(3) @binding(0)
var<uniform> animation: Animation;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let tile_index = f32(animation.tile_index);
	let sprite_size = vec2(0.25, 0.25);
	let column = tile_index % 4.;
	let row = floor(tile_index / 4.);
	let x = (in.tex_coords.x) / 4.;
	let y = (in.tex_coords.y) / 4.;

	let uvOffset = vec2(column * sprite_size.x, row*sprite_size.y);

	let localUv = vec2(x + uvOffset.x, y + uvOffset.y);
	
    return textureSample(texture, texture_sampler, localUv);
}

