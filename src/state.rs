use crate::{
    camera::Camera,
    gpu_buffers::{Buffer as _, DynamicBuffer, FixedSizeBuffer},
};
use encase::{ArrayLength, ShaderSize, ShaderType};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

const RENDER_SAMPLES: u32 = 1;

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuTetrahedron {
    a: cgmath::Vector4<f32>,
    b: cgmath::Vector4<f32>,
    c: cgmath::Vector4<f32>,
    d: cgmath::Vector4<f32>,
}

#[derive(Debug, ShaderType)]
struct GpuTetrahedrons {
    count: ArrayLength,
    #[size(runtime)]
    data: Vec<GpuTetrahedron>,
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuTriangle {
    a: cgmath::Vector3<f32>,
    b: cgmath::Vector3<f32>,
    c: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuIndirectBuffer {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
    /// this is not used by the indirect draw, but for inserting vertices in the compute shader
    pub vertex_count: u32,
}

pub struct State {
    mouse_locked: bool,
    camera: Camera,

    final_texture: wgpu::Texture,

    tetrahedron_count: usize,
    tetrahedron_buffer: DynamicBuffer<GpuTetrahedrons>,
    tetrahedron_triangle_buffer: wgpu::Buffer,
    tetrahedron_index_buffer: wgpu::Buffer,
    tetrahedron_indirect_buffer: FixedSizeBuffer<GpuIndirectBuffer>,
    tetrahedron_bind_group_layout: wgpu::BindGroupLayout,
    tetrahedron_bind_group: wgpu::BindGroup,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> State {
        let camera = Camera::default();
        let final_texture = final_texture(device, 1, 1);

        let tetrahedrons = GpuTetrahedrons {
            count: ArrayLength,
            data: vec![],
        };
        let tetrahedron_count = tetrahedrons.data.len();

        let tetrahedron_buffer = DynamicBuffer::new(
            device,
            queue,
            "Tetrahedrons",
            wgpu::BufferUsages::STORAGE,
            &tetrahedrons,
        );
        let tetrahedron_triangle_buffer = tetrahedron_triangle_buffer(device, tetrahedron_count);
        let tetrahedron_index_buffer = tetrahedron_index_buffer(device, tetrahedron_count);
        let tetrahedron_indirect_buffer = FixedSizeBuffer::new(
            device,
            queue,
            "Tetrahedron Indirect Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            &GpuIndirectBuffer {
                index_count: 0,
                instance_count: 0,
                first_index: 0,
                base_vertex: 0,
                first_instance: 0,
                vertex_count: 0,
            },
        );
        let tetrahedron_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Tetrahedron Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuTetrahedrons::min_size()),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuTriangle::SHADER_SIZE),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: Some(u32::SHADER_SIZE),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuIndirectBuffer::SHADER_SIZE),
                        },
                        count: None,
                    },
                ],
            });
        let tetrahedron_bind_group = tetrahedron_bind_group(
            device,
            &tetrahedron_bind_group_layout,
            tetrahedron_buffer.buffer(),
            &tetrahedron_triangle_buffer,
            &tetrahedron_index_buffer,
            tetrahedron_indirect_buffer.buffer(),
        );

        State {
            camera,
            mouse_locked: false,

            final_texture,
            tetrahedron_count,
            tetrahedron_buffer,
            tetrahedron_triangle_buffer,
            tetrahedron_index_buffer,
            tetrahedron_indirect_buffer,
            tetrahedron_bind_group_layout,
            tetrahedron_bind_group,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let ts = dt.as_secs_f32();

        self.camera.update(ts);
    }

    pub fn key(&mut self, key: KeyCode, state: ElementState, window: &winit::window::Window) {
        if let (KeyCode::Escape, ElementState::Pressed) = (key, state) {
            if self.mouse_locked {
                _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                window.set_cursor_visible(true);
                self.mouse_locked = false;
            } else {
                _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                window.set_cursor_visible(false);
                self.mouse_locked = true;
            }
        }

        self.camera.key(key, state);
    }

    pub fn mouse(&mut self, button: MouseButton, state: ElementState, uv: cgmath::Vector2<f32>) {
        _ = button;
        _ = state;
        _ = uv;
    }

    pub fn focused(&mut self, focused: bool, window: &winit::window::Window) {
        if !focused {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.mouse_locked = false;

            self.camera.reset_keys();
        }
    }

    pub fn mouse_scrolled(&mut self, delta: cgmath::Vector2<f32>) {
        if self.mouse_locked {
            self.camera.mouse_scrolled(delta);
        }
    }

    pub fn mouse_moved(&mut self, delta: cgmath::Vector2<f32>) {
        if self.mouse_locked {
            self.camera.mouse_moved(delta);
        }
    }

    pub fn cursor_moved(&mut self, uv: cgmath::Vector2<f32>) {
        _ = uv;
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.final_texture = final_texture(device, width, height);
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let size @ wgpu::Extent3d { width, height, .. } = texture.size();
        assert_eq!(size, self.final_texture.size());

        _ = width;
        _ = height;

        {
            let tetrahedrons = GpuTetrahedrons {
                count: ArrayLength,
                data: vec![],
            };

            self.tetrahedron_count = tetrahedrons.data.len();
            if self.tetrahedron_buffer.write(device, queue, &tetrahedrons) {
                self.tetrahedron_triangle_buffer =
                    tetrahedron_triangle_buffer(device, self.tetrahedron_count);
                self.tetrahedron_index_buffer =
                    tetrahedron_index_buffer(device, self.tetrahedron_count);
                self.tetrahedron_bind_group = tetrahedron_bind_group(
                    device,
                    &self.tetrahedron_bind_group_layout,
                    self.tetrahedron_buffer.buffer(),
                    &self.tetrahedron_triangle_buffer,
                    &self.tetrahedron_index_buffer,
                    self.tetrahedron_indirect_buffer.buffer(),
                );
            }
        }

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Rendering Encoder"),
        });
        {
            let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self
                        .final_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }
        command_encoder.copy_texture_to_texture(
            self.final_texture.as_image_copy(),
            texture.as_image_copy(),
            size,
        );

        queue.submit(std::iter::once(command_encoder.finish()));
    }
}

fn final_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Final Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: RENDER_SAMPLES,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn tetrahedron_triangle_buffer(device: &wgpu::Device, tetrahedron_count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tetrahedron Triangle Buffer"),
        size: tetrahedron_count.max(1) as wgpu::BufferAddress * GpuTriangle::SHADER_SIZE.get() * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    })
}

fn tetrahedron_index_buffer(device: &wgpu::Device, tetrahedron_count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tetrahedron Index Buffer"),
        size: tetrahedron_count.max(1) as wgpu::BufferAddress * u32::SHADER_SIZE.get() * 6,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
        mapped_at_creation: false,
    })
}

fn tetrahedron_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    tetrahedron_buffer: &wgpu::Buffer,
    triangle_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    indirect_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Tetrahedron Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: tetrahedron_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: triangle_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: index_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: indirect_buffer.as_entire_binding(),
            },
        ],
    })
}
