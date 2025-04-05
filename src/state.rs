use crate::{
    camera::Camera,
    gpu_buffers::{Buffer as _, BufferCreationInfo, BufferGroup, DynamicBuffer, FixedSizeBuffer},
    math::Transform,
};
use encase::{ArrayLength, ShaderSize, ShaderType};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

const RENDER_SAMPLES: u32 = 1;

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuTetrahedron {
    positions: [cgmath::Vector4<f32>; 4],
}

#[derive(Debug, ShaderType)]
struct GpuTetrahedrons {
    count: ArrayLength,
    #[size(runtime)]
    data: Vec<GpuTetrahedron>,
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuVertex {
    position: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuIndirectBuffer {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
    /// this is not used by the indirect draw, but for inserting vertices in the compute shader
    triangle_count: u32,
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct GpuCamera {
    transform: Transform,
    aspect: f32,
}

impl GpuCamera {
    fn from_camera(camera: &Camera, aspect: f32) -> Self {
        Self {
            aspect,
            transform: !(Transform::translation(camera.position)
                * Transform::from_rotor(camera.get_rotation())),
        }
    }
}

pub struct State {
    mouse_locked: bool,
    camera: Camera,
    camera_buffer: BufferGroup<(FixedSizeBuffer<GpuCamera>,)>,

    final_texture: wgpu::Texture,

    tetrahedron_count: usize,
    tetrahedron_buffer: DynamicBuffer<GpuTetrahedrons>,
    tetrahedron_vertex_buffer: wgpu::Buffer,
    tetrahedron_index_buffer: wgpu::Buffer,
    tetrahedron_indirect_buffer: FixedSizeBuffer<GpuIndirectBuffer>,
    tetrahedron_to_triangles_bind_group_layout: wgpu::BindGroupLayout,
    tetrahedron_to_triangles_bind_group: wgpu::BindGroup,
    tetrahedron_to_triangle_compute_pipeline: wgpu::ComputePipeline,
    tetrahedron_triangle_render_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> State {
        let camera = Camera::default();
        let camera_buffer = BufferGroup::new(
            device,
            "Camera",
            (BufferCreationInfo {
                buffer: FixedSizeBuffer::new(
                    device,
                    queue,
                    "Camera Buffer",
                    wgpu::BufferUsages::UNIFORM,
                    &GpuCamera::from_camera(&camera, 1.0),
                ),
                binding_type: wgpu::BufferBindingType::Uniform,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
            },),
        );

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
        let tetrahedron_vertex_buffer = tetrahedron_triangle_buffer(device, tetrahedron_count);
        let tetrahedron_index_buffer = tetrahedron_index_buffer(device, tetrahedron_count);
        let tetrahedron_indirect_buffer = FixedSizeBuffer::new(
            device,
            queue,
            "Tetrahedron Indirect Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            &GpuIndirectBuffer {
                index_count: 0,
                instance_count: 1,
                first_index: 0,
                base_vertex: 0,
                first_instance: 0,
                triangle_count: 0,
            },
        );
        let tetrahedron_to_triangles_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Tetrahedron To Triangles Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuIndirectBuffer::SHADER_SIZE),
                        },
                        count: None,
                    },
                ],
            });
        let tetrahedron_to_triangles_bind_group = tetrahedron_to_triangles_bind_group(
            device,
            &tetrahedron_to_triangles_bind_group_layout,
            tetrahedron_buffer.buffer(),
            &tetrahedron_vertex_buffer,
            &tetrahedron_index_buffer,
            tetrahedron_indirect_buffer.buffer(),
        );

        let tetrahedron_to_triangle_shader = device.create_shader_module(wgpu::include_wgsl!(
            "./shaders/tetrahedrons_to_triangles.wgsl"
        ));
        let tetrahedron_to_triangle_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tetrahedron To Triangle Pipeline Layout"),
                bind_group_layouts: &[
                    camera_buffer.bind_group_layout(),
                    &tetrahedron_to_triangles_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let tetrahedron_to_triangle_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Tetrahedron To Triangle Compute Pipeline"),
                layout: Some(&tetrahedron_to_triangle_pipeline_layout),
                module: &tetrahedron_to_triangle_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: Default::default(),
            });

        let triangle_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/triangles.wgsl"));
        let tetrahedron_triangle_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tetrahedron Triangle Pipeline Layout"),
                bind_group_layouts: &[camera_buffer.bind_group_layout()],
                push_constant_ranges: &[],
            });
        let tetrahedron_triangle_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Tetrahedron Triangle Render Pipeline"),
                layout: Some(&tetrahedron_triangle_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &triangle_shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: GpuVertex::SHADER_SIZE.get(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    // TODO: maybe use this?
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: RENDER_SAMPLES,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &triangle_shader,
                    entry_point: Some("pixel"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: Default::default(),
            });

        State {
            mouse_locked: false,
            camera,
            camera_buffer,

            final_texture,
            tetrahedron_count,
            tetrahedron_buffer,
            tetrahedron_vertex_buffer,
            tetrahedron_index_buffer,
            tetrahedron_indirect_buffer,
            tetrahedron_to_triangles_bind_group_layout,
            tetrahedron_to_triangles_bind_group,
            tetrahedron_to_triangle_compute_pipeline,
            tetrahedron_triangle_render_pipeline,
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

        // upload
        {
            self.camera_buffer.write(
                device,
                queue,
                (Some(&GpuCamera::from_camera(
                    &self.camera,
                    width as f32 / height as f32,
                )),),
            );

            // tetrahedrons
            {
                let tetrahedrons = GpuTetrahedrons {
                    count: ArrayLength,
                    data: vec![GpuTetrahedron {
                        positions: [
                            cgmath::vec4(1.0, 0.5, 0.0, 0.0),
                            cgmath::vec4(1.0, -0.5, 0.5, 1.0),
                            cgmath::vec4(1.0, -0.5, -0.5, -1.0),
                            cgmath::vec4(2.0, 0.0, 1.0, 0.0),
                        ],
                    }],
                };

                self.tetrahedron_count = tetrahedrons.data.len();
                if self.tetrahedron_buffer.write(device, queue, &tetrahedrons) {
                    self.tetrahedron_vertex_buffer =
                        tetrahedron_triangle_buffer(device, self.tetrahedron_count);
                    self.tetrahedron_index_buffer =
                        tetrahedron_index_buffer(device, self.tetrahedron_count);
                    self.tetrahedron_to_triangles_bind_group = tetrahedron_to_triangles_bind_group(
                        device,
                        &self.tetrahedron_to_triangles_bind_group_layout,
                        self.tetrahedron_buffer.buffer(),
                        &self.tetrahedron_vertex_buffer,
                        &self.tetrahedron_index_buffer,
                        self.tetrahedron_indirect_buffer.buffer(),
                    );
                }
            }

            // reset triangle indirect buffer
            self.tetrahedron_indirect_buffer.write(
                device,
                queue,
                &GpuIndirectBuffer {
                    index_count: 0,
                    instance_count: 1,
                    first_index: 0,
                    base_vertex: 0,
                    first_instance: 0,
                    triangle_count: 0,
                },
            );

            queue.submit(std::iter::empty());
        }

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Rendering Encoder"),
        });

        // compute triangles from tetrahedrons
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Tetrahedron To Triangle Compute Pass"),
                    timestamp_writes: None,
                });

            compute_pass.set_pipeline(&self.tetrahedron_to_triangle_compute_pipeline);
            compute_pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);
            compute_pass.set_bind_group(1, &self.tetrahedron_to_triangles_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.tetrahedron_count as _, 1, 1);
        }

        // render to final texture
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self
                        .final_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.tetrahedron_triangle_render_pipeline);
            render_pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.tetrahedron_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.tetrahedron_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed_indirect(self.tetrahedron_indirect_buffer.buffer(), 0);
        }

        // copy final texture to screen
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
        size: tetrahedron_count.max(1) as wgpu::BufferAddress * GpuVertex::SHADER_SIZE.get() * 4,
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

fn tetrahedron_to_triangles_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    tetrahedron_buffer: &wgpu::Buffer,
    triangle_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    indirect_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Tetrahedron To Triangles Bind Group"),
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
