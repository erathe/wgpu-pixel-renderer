use wgpu_tilemap_renderer::run;

fn main() {
    pollster::block_on(run());
}
