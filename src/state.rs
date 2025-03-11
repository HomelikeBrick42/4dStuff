use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

use crate::{
    camera::Camera,
    gpu_buffers::{ArrayBuffer, BufferCreationInfo, BufferGroup, FixedSizeBuffer},
    gpu_types::{GpuCamera, GpuHyperSphere, GpuMaterial},
    hyper_sphere::HyperSphere,
    material::Material,
};

pub struct State {
    main_texture_output_bind_group_layout: wgpu::BindGroupLayout,
    main_texture_render_bind_group_layout: wgpu::BindGroupLayout,
    main_texture: wgpu::Texture,
    main_texture_output_bind_group: wgpu::BindGroup,
    main_texture_render_bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_buffer: BufferGroup<(FixedSizeBuffer<GpuCamera>,)>,

    materials: Vec<Material>,
    hyper_spheres: Vec<HyperSphere>,
    objects_buffer: BufferGroup<(ArrayBuffer<GpuMaterial>, ArrayBuffer<GpuHyperSphere>)>,

    ray_tracing_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    mouse_locked: bool,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> State {
        let (main_texture_output_bind_group_layout, main_texture_render_bind_group_layout) =
            main_texture_bind_group_layouts(device);
        let (main_texture, main_texture_output_bind_group, main_texture_render_bind_group) =
            main_texture_and_bind_groups(
                device,
                1,
                1,
                &main_texture_output_bind_group_layout,
                &main_texture_render_bind_group_layout,
            );

        let camera = Camera::default();
        let camera_buffer = BufferGroup::new(
            device,
            "Camera",
            (BufferCreationInfo {
                buffer: FixedSizeBuffer::new(
                    device,
                    queue,
                    "Camera",
                    wgpu::BufferUsages::UNIFORM,
                    &GpuCamera::from_camera(&camera),
                ),
                binding_type: wgpu::BufferBindingType::Uniform,
                visibility: wgpu::ShaderStages::COMPUTE,
            },),
        );

        let materials = vec![
            Material {
                color: cgmath::vec3(0.2, 0.6, 0.3),
            },
            Material {
                color: cgmath::vec3(0.8, 0.2, 0.1),
            },
        ];
        let hyper_spheres = vec![
            HyperSphere {
                position: cgmath::vec4(3.0, -1001.0, 0.0, 0.0),
                radius: 1000.0,
                material: 0,
            },
            HyperSphere {
                position: cgmath::vec4(3.0, 0.0, 0.0, 0.0),
                radius: 1.0,
                material: 1,
            },
        ];
        let objects_buffer = BufferGroup::new(
            device,
            "Objects",
            (
                BufferCreationInfo {
                    buffer: ArrayBuffer::new(
                        device,
                        queue,
                        "Materials",
                        wgpu::BufferUsages::STORAGE,
                        materials
                            .iter()
                            .map(GpuMaterial::from_material)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                    binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                    visibility: wgpu::ShaderStages::COMPUTE,
                },
                BufferCreationInfo {
                    buffer: ArrayBuffer::new(
                        device,
                        queue,
                        "Hyper Spheres",
                        wgpu::BufferUsages::STORAGE,
                        hyper_spheres
                            .iter()
                            .map(GpuHyperSphere::from_hyper_sphere)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                    binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                    visibility: wgpu::ShaderStages::COMPUTE,
                },
            ),
        );

        let ray_tracing_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/ray_tracing.wgsl"));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &main_texture_output_bind_group_layout,
                    camera_buffer.bind_group_layout(),
                    objects_buffer.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let full_screen_quad_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/full_screen_quad.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&main_texture_render_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            vertex: wgpu::VertexState {
                module: &full_screen_quad_shader,
                entry_point: Some("vertex"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &full_screen_quad_shader,
                entry_point: Some("fragment"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        State {
            main_texture_output_bind_group_layout,
            main_texture_render_bind_group_layout,
            main_texture,
            main_texture_output_bind_group,
            main_texture_render_bind_group,

            camera,
            camera_buffer,

            materials,
            hyper_spheres,
            objects_buffer,

            ray_tracing_pipeline,
            render_pipeline,

            mouse_locked: false,
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

    pub fn mouse(&mut self, _button: MouseButton, _state: ElementState) {}

    pub fn focused(&mut self, focused: bool, window: &winit::window::Window) {
        if !focused {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.mouse_locked = false;

            self.camera.reset_keys();
        }
    }

    pub fn mouse_moved(&mut self, delta: cgmath::Vector2<f32>) {
        if self.mouse_locked {
            self.camera.mouse_moved(delta);
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (
            self.main_texture,
            self.main_texture_output_bind_group,
            self.main_texture_render_bind_group,
        ) = main_texture_and_bind_groups(
            device,
            width,
            height,
            &self.main_texture_output_bind_group_layout,
            &self.main_texture_render_bind_group_layout,
        );
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        self.camera_buffer.write(
            device,
            queue,
            (Some(&GpuCamera::from_camera(&self.camera)),),
        );
        self.objects_buffer.write(
            device,
            queue,
            (
                Some(
                    self.materials
                        .iter()
                        .map(GpuMaterial::from_material)
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
                Some(
                    self.hyper_spheres
                        .iter()
                        .map(GpuHyperSphere::from_hyper_sphere)
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
            ),
        );

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Rendering Encoder"),
        });
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Ray Tracing Pass"),
                    timestamp_writes: None,
                });

            let wgpu::Extent3d { width, height, .. } = self.main_texture.size();

            compute_pass.set_pipeline(&self.ray_tracing_pipeline);
            compute_pass.set_bind_group(0, &self.main_texture_output_bind_group, &[]);
            compute_pass.set_bind_group(1, self.camera_buffer.bind_group(), &[]);
            compute_pass.set_bind_group(2, self.objects_buffer.bind_group(), &[]);
            compute_pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
        }
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.main_texture_render_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }
        queue.submit(std::iter::once(command_encoder.finish()));
    }
}

fn main_texture_bind_group_layouts(
    device: &wgpu::Device,
) -> (wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
    let output_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main Texture Output Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
    let render_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main Texture Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
    (output_bind_group_layout, render_bind_group_layout)
}

fn main_texture_and_bind_groups(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    output_layout: &wgpu::BindGroupLayout,
    render_layout: &wgpu::BindGroupLayout,
) -> (wgpu::Texture, wgpu::BindGroup, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Main Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let output_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Main Texture Output Bind Group"),
        layout: output_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
    });
    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Main Texture Render Bind Group"),
        layout: render_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (texture, output_bind_group, render_bind_group)
}
