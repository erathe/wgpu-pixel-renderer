@group(0) @binding(0) var source_texture: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var dest_texture: texture_storage_2d<r32float, write>;
@group(1) @binding(0) var<uniform> params: Params;
struct Params {
    texture_size: vec2<u32>,
    jump_distance: u32,
    run: i32,
};

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let id: vec2<i32> = vec2<i32>(GlobalInvocationID.xy);
    var min_distance: f32 = f32(params.texture_size.x) * f32(params.texture_size.y); // Initial high value
    let step = i32(params.jump_distance);

    // Loop to sample surrounding points based on the jump distance
	// TODO: rename things to make everything more clear.
	// TODO: See if there is a better way to do for loops in shaders
    for (var dy: i32 = -step; dy <= step; dy += step) {
        for (var dx: i32 = -step; dx <= step; dx += step) {
            // if (params.run > 8 && dx == 0 && dy == 0) {
            //     continue; // Skip the current pixel's position
            // }
            let samplePos: vec2<i32> = id + vec2<i32>(dx, dy);

            // Ensure we're within texture bounds
            if all(samplePos >= vec2<i32>(0)) && all(samplePos < vec2<i32>(params.texture_size)) {
                // Load the distance from the occluder texture at the sample position
                let sample_distance: f32 = textureLoad(source_texture, samplePos).r;

                // Calculate Euclidean distance from the current pixel to the sample position,
                // converting dx and dy to floats for the distance calculation
                let spatial_distance: f32 = length(vec2<f32>(f32(dx), f32(dy)));

                // Sum the spatial distance and the sampled distance
                let distance_to_sample: f32 = spatial_distance + sample_distance;

                // Update min_distance if this path is shorter
                min_distance = min(min_distance, distance_to_sample);
            }
        }
    }

    // Write the updated minimum distance to the SDF texture
    textureStore(dest_texture, vec2<i32>(id), vec4<f32>(min_distance, 0.0, 0.0, 1.0));
}
