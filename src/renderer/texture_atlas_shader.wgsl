
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

@group(3) @binding(0)
var<storage, read> lights: array<Light>;

struct Light {
	position: vec2<f32>,
	intensity: f32,
	falloff: f32,
	color: vec3<f32>,
};


//TODO: uniform
const screen = vec2(1920., 1200.);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	// sprite
	let sprite_size = in.size / atlas.size;
	let uvOffset = in.tex_coords * sprite_size;
	let localUv = uvOffset + (in.texture_origin / atlas.size);
	
	var base_sample = textureSample(texture, texture_sampler, localUv);

	// lighting
	// TODO: Should probably be a post processing step
	let w_p = in.world_position;

	// where to sample the sdf
	let world_uv = w_p / screen;

	// make everything dark
	let ambient_light = vec3(0.015, 0.015, 0.015);
	var final_color = base_sample.rgb * ambient_light;

	// TODO: add num lights as a uniform
	for (var i: i32 = 0; i < 10; i = i + 1) {
		let light = lights[i];

	 	let light_dir = normalize(light.position - w_p);
		let dist = length(light.position - w_p);
		var dist_traveled = 0.0;
		var reached = true;

		// soft shadows
		var s = 1.0;
		var k = 12.0;

		// raymarch sample lights
		// can't hard core arrays in wgsl so just do it twice until I put stuff in uniforms
		for (var j: i32 = 0; j < 100; j = j + 1) {
			let d = textureSample(sdf_texture, texture_sampler, (w_p + (dist_traveled * light_dir)) / screen).r;

			if (d < 0.00001) {
				reached = false;
				break;
			}

			dist_traveled += d;
			s = min(s, (k * d) / dist_traveled);
			if (dist_traveled >= dist) {
				break;
			};
		}

		if (reached) {
			let falloff = (light.intensity * 100.) / (1.0 + (dist * dist * light.falloff));
			final_color += (base_sample.rgb * light.color) * falloff * s;
		}
	}	

	return vec4(final_color, base_sample.a);
}
