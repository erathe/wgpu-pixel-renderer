A 2D tilemap renderer with dynamic ray traced lighting

Basic overview:
- Collects tiles to render from a string.
- Uses the same string to create an occluder texture (currently a naive approach, only supporting square, static occluders).
- Generates a sdf texture from the occluder texture using a compute shader and the jump-flood algorithm over multiple passes with
  alternating textures.
- Uses the sdf texture to ray march all the lights in the scene and calculate the combined light contribution for each fragment

low-res video of the running program:
https://github.com/erathe/wgpu-pixel-renderer/assets/789055/34133011-3e8d-4c93-9ee1-f8f276d58984

higher resolution:
https://www.dropbox.com/scl/fi/qa884913pxeklmiau4hji/Screen-Recording-2024-04-03-at-22.13.00.mov?rlkey=xv8vdjl5qvpb3dbhgl18k0pqx&dl=0
