use wgpu_tilemap_renderer::run;
mod renderer;

fn main() {
    pollster::block_on(run());
}
