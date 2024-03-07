use cgmath::Matrix4;
use wgpu::util::DeviceExt;

use super::utils::{PipelineData, UniformData};

const Z_RANGE: f32 = 100.;

pub struct Camera {
    height: f32,
    width: f32,
    scale: f32,
    offset: (f32, f32),
    proj: Matrix4<f32>,
    pub uniform: UniformData<CameraUniform>,
    pub pipeline_data: PipelineData,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        height: f32,
        width: f32,
        scale: f32,
        offset: (f32, f32),
    ) -> Self {
        let (x, y) = offset;
        let w = width / scale;
        let h = height / scale;
        let proj = cgmath::ortho(
            (-w / 2.0) + x,
            (w / 2.0) + x,
            -(h / 2.0) + y,
            (h / 2.0) + y,
            -Z_RANGE,
            Z_RANGE,
        );

        let uniform = CameraUniform {
            view_proj: proj.into(),
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layou"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            label: Some("Camera Bind Group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let pipeline_data = PipelineData::new(bind_group, bind_group_layout);

        let camera_uniform = UniformData::new(uniform, buffer);

        Self {
            height,
            width,
            scale,
            proj,
            uniform: camera_uniform,
            pipeline_data,
            offset,
        }
    }

    pub fn update_view_projection(&mut self, n_h: f32, n_w: f32, d_s: f32) {
        self.scale += d_s;
        self.height = n_h;
        self.width = n_w;
        let (x, y) = self.offset;
        let w = self.width / self.scale;
        let h = self.height / self.scale;
        self.proj = cgmath::ortho(
            (-w / 2.0) + x,
            (w / 2.0) + x,
            -(h / 2.0) + y,
            (h / 2.0) + y,
            -Z_RANGE,
            Z_RANGE,
        );
        self.update_view_proj_uniform();
    }

    pub fn move_camera(&mut self, offset: (f32, f32)) {
        self.offset = offset;
        let (w, h) = (self.width, self.height);
        let (x, y) = self.offset;
        self.proj = cgmath::ortho(
            (-w / 2.0) + x,
            (w / 2.0) + x,
            -(h / 2.0) + y,
            (h / 2.0) + y,
            -Z_RANGE,
            Z_RANGE,
        );
        self.update_view_proj_uniform();
    }

    pub fn update_view_proj_uniform(&mut self) {
        self.uniform.uniform.view_proj = self.proj.into()
    }

    pub fn get_uniform(&self) -> &UniformData<CameraUniform> {
        &self.uniform
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.pipeline_data.bind_group()
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.pipeline_data.bind_group_layout()
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}
