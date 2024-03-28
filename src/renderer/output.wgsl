struct VertexInput {
    @location(0) position: vec2<f32>,
	@location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	let position = vec4(input.position, 0.0, 1.0);
	out.clip_position = position;
	out.tex_coords = input.tex_coords;
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	var sampled = textureSample(texture, texture_sampler, in.tex_coords);
	// sampled = sampled * vec4(0.10, 0.10, 0.10, 1.0);
    return sampled;
}
