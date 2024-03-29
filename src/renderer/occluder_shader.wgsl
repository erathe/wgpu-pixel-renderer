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
	out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
	out.tex_coords = input.tex_coords;
	// out.tex_coords.y = 1. - out.tex_coords.y;
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let max_d = 600.;
	let sample = textureSample(texture, texture_sampler, in.tex_coords).r;
	let normalized = sample / max_d;

	// let smooth_v: f32 = smoothstep(0.0, 1.0, normalized);

	return vec4<f32>(vec3<f32>(normalized), 1.0);

	// return vec4<f32>(vec3<f32>(smooth_v), 1.0);
	// let color: vec4<f32> = textureSample(texture, texture_sampler, in.tex_coords);

 //    return color;
}
