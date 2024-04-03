A 2D tilemap renderer with dynamic ray traced lighting

Basic overview:
- collects tiles to render from a string
- uses the same string to create an occluder texture (currently a naive approach)
- generates a sdf texture from the occluder texture using a compute shader and the jump-flood algorithm over multiple passes
- uses the sdf texture to ray march all the lights in the scene and calculate the light contribution for each fragment

low-res video of the running program:
https://github.com/erathe/wgpu-pixel-renderer/assets/789055/34133011-3e8d-4c93-9ee1-f8f276d58984

