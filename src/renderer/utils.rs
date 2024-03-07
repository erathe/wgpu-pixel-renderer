pub struct UniformData<T> {
    pub uniform: T,
    pub buffer: wgpu::Buffer,
}

impl<T> UniformData<T> {
    pub fn new(uniform: T, buffer: wgpu::Buffer) -> Self {
        Self { uniform, buffer }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

pub struct PipelineData {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl PipelineData {
    pub fn new(bind_group: wgpu::BindGroup, bind_group_layout: wgpu::BindGroupLayout) -> Self {
        Self {
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

pub fn to_linear_rgb(c: u32) -> [f32; 3] {
    let f = |xu: u32| {
        let x = (xu & 0xFF) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c >> 16), f(c >> 8), f(c)]
}
