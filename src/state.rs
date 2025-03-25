use crate::{
    camera::Camera,
    gpu_buffers::{BufferCreationInfo, BufferGroup, DynamicBuffer, FixedSizeBuffer},
    gpu_types::{
        GpuCamera, GpuHyperPlane, GpuHyperSphere, GpuLengthArray, GpuLine, GpuMaterial, GpuUiInfo,
    },
    material::Material,
    math::{Rotor, Transform},
    objects::{HyperPlane, HyperSphere, Object},
    ray::{Ray, RayIntersect},
};
use cgmath::InnerSpace;
use encase::ArrayLength;
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

const RENDER_SAMPLES: u32 = 4;

pub struct State {
    camera: Camera,
    camera_buffer: BufferGroup<(FixedSizeBuffer<GpuCamera>,)>,

    materials: Vec<Material>,
    objects: Vec<Object>,
    #[expect(clippy::type_complexity)]
    objects_buffer: BufferGroup<(
        DynamicBuffer<Vec<GpuMaterial>>,
        DynamicBuffer<GpuLengthArray<GpuHyperSphere>>,
        DynamicBuffer<GpuLengthArray<GpuHyperPlane>>,
    )>,

    ui_buffer: BufferGroup<(FixedSizeBuffer<GpuUiInfo>, DynamicBuffer<Vec<GpuLine>>)>,

    ray_tracing_texture_output_bind_group_layout: wgpu::BindGroupLayout,
    ray_tracing_texture_render_bind_group_layout: wgpu::BindGroupLayout,
    ray_tracing_texture: wgpu::Texture,
    ray_tracing_texture_output_bind_group: wgpu::BindGroup,
    ray_tracing_texture_render_bind_group: wgpu::BindGroup,
    ray_tracing_pipeline: wgpu::ComputePipeline,

    ray_tracing_render_pipeline: wgpu::RenderPipeline,
    ui_render_pipeline: wgpu::RenderPipeline,

    final_texture: wgpu::Texture,

    selected_hyper_sphere: Option<usize>,
    axis_line_interaction: Option<AxisLineInteraction>,
    use_camera_axes: bool,
    mouse_locked: bool,
}

struct AxisLineInteraction {
    axis_index: usize,
    start_pos: f32,
    dragging: bool,
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
                color: cgmath::vec3(0.1, 0.6, 0.2),
            },
            Material {
                color: cgmath::vec3(0.8, 0.2, 0.1),
            },
            Material {
                color: cgmath::vec3(0.2, 0.8, 0.3),
            },
            Material {
                color: cgmath::vec3(0.1, 0.2, 0.8),
            },
        ];
        let objects = vec![
            Object::HyperPlane(HyperPlane {
                position: cgmath::vec4(0.0, -1.0, 0.0, 0.0),
                normal: cgmath::vec4(0.0, 1.0, 0.0, 0.0),
                material: 0,
            }),
            Object::HyperSphere(HyperSphere {
                position: cgmath::vec4(3.0, 0.0, 0.0, 0.0),
                radius: 1.0,
                material: 1,
            }),
            Object::HyperSphere(HyperSphere {
                position: cgmath::vec4(3.0, 0.0, 2.0, 0.0),
                radius: 1.0,
                material: 2,
            }),
            Object::HyperSphere(HyperSphere {
                position: cgmath::vec4(3.0, 0.0, -2.0, 2.0),
                radius: 1.0,
                material: 3,
            }),
        ];
        let objects_buffer = {
            let (hyper_spheres, hyper_planes) = Self::objects_to_gpu_objects(&objects);
            BufferGroup::new(
                device,
                "Objects",
                (
                    BufferCreationInfo {
                        buffer: DynamicBuffer::new(
                            device,
                            queue,
                            "Materials",
                            wgpu::BufferUsages::STORAGE,
                            &materials
                                .iter()
                                .map(GpuMaterial::from_material)
                                .collect::<Vec<_>>(),
                        ),
                        binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                        visibility: wgpu::ShaderStages::COMPUTE,
                    },
                    BufferCreationInfo {
                        buffer: DynamicBuffer::new(
                            device,
                            queue,
                            "Hyper Spheres",
                            wgpu::BufferUsages::STORAGE,
                            &GpuLengthArray {
                                length: ArrayLength,
                                data: hyper_spheres,
                            },
                        ),
                        binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                        visibility: wgpu::ShaderStages::COMPUTE,
                    },
                    BufferCreationInfo {
                        buffer: DynamicBuffer::new(
                            device,
                            queue,
                            "Hyper Planes",
                            wgpu::BufferUsages::STORAGE,
                            &GpuLengthArray {
                                length: ArrayLength,
                                data: hyper_planes,
                            },
                        ),
                        binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                        visibility: wgpu::ShaderStages::COMPUTE,
                    },
                ),
            )
        };

        let ui_buffer = BufferGroup::new(
            device,
            "UI",
            (
                BufferCreationInfo {
                    buffer: FixedSizeBuffer::new(
                        device,
                        queue,
                        "Info",
                        wgpu::BufferUsages::UNIFORM,
                        &GpuUiInfo { aspect: 1.0 },
                    ),
                    binding_type: wgpu::BufferBindingType::Uniform,
                    visibility: wgpu::ShaderStages::VERTEX,
                },
                BufferCreationInfo {
                    buffer: DynamicBuffer::new(
                        device,
                        queue,
                        "Lines",
                        wgpu::BufferUsages::STORAGE,
                        &vec![],
                    ),
                    binding_type: wgpu::BufferBindingType::Storage { read_only: true },
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                },
            ),
        );

        let (
            ray_tracing_texture_output_bind_group_layout,
            ray_tracing_texture_render_bind_group_layout,
        ) = ray_tracing_texture_bind_group_layouts(device);
        let (
            ray_tracing_texture,
            ray_tracing_texture_output_bind_group,
            ray_tracing_texture_render_bind_group,
        ) = ray_tracing_texture_and_bind_groups(
            device,
            1,
            1,
            &ray_tracing_texture_output_bind_group_layout,
            &ray_tracing_texture_render_bind_group_layout,
        );

        let ray_tracing_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/ray_tracing.wgsl"));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &ray_tracing_texture_output_bind_group_layout,
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
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Render Pipeline Layout"),
                bind_group_layouts: &[&ray_tracing_texture_render_bind_group_layout],
                push_constant_ranges: &[],
            });
        let ray_tracing_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Ray Tracing Render Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
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
                    count: RENDER_SAMPLES,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let ui_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/lines.wgsl"));
        let ui_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[ui_buffer.bind_group_layout()],
            push_constant_ranges: &[],
        });
        let ui_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&ui_pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: Some("vertex"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: Some("fragment"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: RENDER_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let final_texture = final_texture(
            device,
            ray_tracing_texture.width(),
            ray_tracing_texture.height(),
        );

        State {
            camera,
            camera_buffer,

            materials,
            objects,
            objects_buffer,

            ui_buffer,

            ray_tracing_texture_output_bind_group_layout,
            ray_tracing_texture_render_bind_group_layout,
            ray_tracing_texture,
            ray_tracing_texture_output_bind_group,
            ray_tracing_texture_render_bind_group,
            ray_tracing_pipeline,

            ray_tracing_render_pipeline,
            ui_render_pipeline,

            final_texture,

            selected_hyper_sphere: None,
            axis_line_interaction: None,
            use_camera_axes: false,
            mouse_locked: false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let ts = dt.as_secs_f32();

        self.camera.update(ts);
    }

    pub fn key(&mut self, key: KeyCode, state: ElementState, window: &winit::window::Window) {
        match (key, state) {
            (KeyCode::Escape, ElementState::Pressed) => {
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

            (KeyCode::KeyG, ElementState::Pressed) => {
                self.use_camera_axes = !self.use_camera_axes;
                if let Some(interaction) = &mut self.axis_line_interaction {
                    interaction.dragging = false;
                }
            }

            _ => (),
        }

        self.camera.key(key, state);
    }

    pub fn mouse(&mut self, button: MouseButton, state: ElementState, uv: cgmath::Vector2<f32>) {
        if !self.mouse_locked {
            match (button, state) {
                (MouseButton::Left, ElementState::Released) => self.axis_line_interaction = None,

                (MouseButton::Left, ElementState::Pressed) => {
                    if let Some(interaction) = &mut self.axis_line_interaction {
                        interaction.dragging = true;
                    }

                    if self.axis_line_interaction.is_none() {
                        let rotation = self.camera.get_rotation();
                        let forward = rotation.rotate(Camera::FORWARD);
                        let up = rotation.rotate(Camera::UP);
                        let right = rotation.rotate(Camera::RIGHT);

                        let ray = Ray {
                            origin: self.camera.position,
                            direction: (right * uv.x + up * uv.y + forward).normalize(),
                        };
                        let hit = self.objects.iter().enumerate().fold(
                            None,
                            |current_hit, (index, hyper_sphere)| {
                                let hit = hyper_sphere.intersect(ray);
                                match (current_hit, hit) {
                                    (None, None) => None,
                                    (None, Some(hit)) => Some((index, hit)),
                                    (Some(_), None) => current_hit,
                                    (Some((current_index, current_hit)), Some(hit)) => {
                                        if current_hit.distance < hit.distance {
                                            Some((current_index, current_hit))
                                        } else {
                                            Some((index, hit))
                                        }
                                    }
                                }
                            },
                        );

                        println!("{hit:?}");

                        if let Some((index, _)) = hit {
                            self.selected_hyper_sphere = Some(index);
                        } else {
                            self.selected_hyper_sphere = None;
                        }
                    }
                }

                _ => {}
            }
        }
    }

    pub fn focused(&mut self, focused: bool, window: &winit::window::Window) {
        if !focused {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.mouse_locked = false;
            self.axis_line_interaction = None;

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
        if !self.mouse_locked {
            if let Some(selected_hyper_sphere) = self.selected_hyper_sphere {
                'axis_lines: {
                    let axis_lines = Self::get_axis_lines(
                        &self.camera,
                        &self.objects[selected_hyper_sphere],
                        self.use_camera_axes,
                    );

                    let axis_lines = axis_lines.map(|axis_line| {
                        axis_line.and_then(|(axis_line, _)| {
                            let a_to_b = axis_line.b - axis_line.a;
                            let line_length = a_to_b.magnitude();
                            if line_length < 0.01 {
                                return None;
                            }
                            let a_to_b = a_to_b / line_length;
                            let normal = cgmath::vec2(a_to_b.y, -a_to_b.x);

                            let pos = (uv - axis_line.a).dot(a_to_b) / line_length;
                            let dist = (uv - axis_line.a).dot(normal).abs();

                            if dist > axis_line.width * 4.0
                                && self
                                    .axis_line_interaction
                                    .as_ref()
                                    .is_none_or(|interaction| !interaction.dragging)
                            {
                                return None;
                            }

                            Some((pos, dist))
                        })
                    });

                    if let Some(interaction) = &mut self.axis_line_interaction {
                        if interaction.dragging {
                            if let Some((pos, _)) = axis_lines[interaction.axis_index] {
                                let object = &mut self.objects[self.selected_hyper_sphere.expect(
                                    "if an axis is being dragged a hyper sphere must be selected",
                                )];
                                let axis = Self::axis_from_index(
                                    interaction.axis_index,
                                    self.use_camera_axes.then(|| self.camera.get_rotation()),
                                );
                                let pos_delta = pos - interaction.start_pos;
                                object.move_position(axis * pos_delta);
                                break 'axis_lines;
                            }
                        }
                    }

                    let closest = axis_lines
                        .into_iter()
                        .enumerate()
                        .filter_map(|(index, distances)| {
                            distances.and_then(|(pos, dist)| {
                                (0.0..=1.0).contains(&pos).then_some((index, pos, dist))
                            })
                        })
                        .min_by(|(_, _, a_dist), (_, _, b_dist)| a_dist.total_cmp(b_dist));

                    if let Some((index, pos, _)) = closest {
                        self.axis_line_interaction = Some(AxisLineInteraction {
                            axis_index: index,
                            start_pos: pos,
                            dragging: false,
                        });
                    } else {
                        self.axis_line_interaction = None;
                    }
                }
            }
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (
            self.ray_tracing_texture,
            self.ray_tracing_texture_output_bind_group,
            self.ray_tracing_texture_render_bind_group,
        ) = ray_tracing_texture_and_bind_groups(
            device,
            width,
            height,
            &self.ray_tracing_texture_output_bind_group_layout,
            &self.ray_tracing_texture_render_bind_group_layout,
        );

        self.final_texture = final_texture(device, width, height);
    }

    fn axis_from_index(index: usize, rotation: Option<Rotor>) -> cgmath::Vector4<f32> {
        let axis = match index {
            0 => cgmath::vec4(1.0, 0.0, 0.0, 0.0),
            1 => cgmath::vec4(0.0, 1.0, 0.0, 0.0),
            2 => cgmath::vec4(0.0, 0.0, 1.0, 0.0),
            3 => cgmath::vec4(0.0, 0.0, 0.0, 1.0),
            _ => panic!("invalid index"),
        };
        if let Some(rotation) = rotation {
            rotation.rotate(axis)
        } else {
            axis
        }
    }

    fn get_axis_lines(
        camera: &Camera,
        object: &Object,
        use_camera_axes: bool,
    ) -> [Option<(GpuLine, f32)>; 4] {
        let camera_transform =
            Transform::translation(camera.position) * Transform::from_rotor(camera.get_rotation());

        // applying the inverse camera transform to the position
        let object_position = object.position();
        let position = (!camera_transform).transform(object_position);
        if position.x >= 0.0 {
            let position = cgmath::vec2(position.z / position.x, position.y / position.x);

            let axis_lines = {
                let rotation = use_camera_axes.then_some(camera_transform.rotor_part());
                [
                    (
                        Self::axis_from_index(0, rotation),
                        cgmath::vec4(1.0, 0.2, 0.2, 1.0),
                    ),
                    (
                        Self::axis_from_index(1, rotation),
                        cgmath::vec4(0.2, 1.0, 0.2, 1.0),
                    ),
                    (
                        Self::axis_from_index(2, rotation),
                        cgmath::vec4(0.2, 0.2, 1.0, 1.0),
                    ),
                    (
                        Self::axis_from_index(3, rotation),
                        cgmath::vec4(1.0, 0.2, 1.0, 1.0),
                    ),
                ]
            };

            axis_lines.map(|(axis_offset, axis_color)| {
                let end_point = (!camera_transform).transform(object_position + axis_offset);
                if end_point.x >= 0.0 {
                    Some((
                        GpuLine {
                            a: position,
                            b: cgmath::vec2(end_point.z / end_point.x, end_point.y / end_point.x),
                            width: 0.01,
                            color: axis_color,
                        },
                        end_point.x,
                    ))
                } else {
                    None
                }
            })
        } else {
            [const { None }; 4]
        }
    }

    fn objects_to_gpu_objects(objects: &[Object]) -> (Vec<GpuHyperSphere>, Vec<GpuHyperPlane>) {
        let mut hyper_spheres = vec![];
        let mut hyper_planes = vec![];
        for object in objects {
            match object {
                Object::HyperSphere(hyper_sphere) => {
                    hyper_spheres.push(GpuHyperSphere::from_hyper_sphere(hyper_sphere));
                }
                Object::HyperPlane(hyper_plane) => {
                    hyper_planes.push(GpuHyperPlane::from_hyper_plane(hyper_plane));
                }
            }
        }
        (hyper_spheres, hyper_planes)
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let wgpu::Extent3d { width, height, .. } = texture.size();
        assert_eq!(texture.size(), self.ray_tracing_texture.size());
        assert_eq!(texture.size(), self.final_texture.size());

        self.camera_buffer.write(
            device,
            queue,
            (Some(&GpuCamera::from_camera(&self.camera)),),
        );

        {
            let (hyper_spheres, hyper_planes) = Self::objects_to_gpu_objects(&self.objects);
            self.objects_buffer.write(
                device,
                queue,
                (
                    Some(
                        &self
                            .materials
                            .iter()
                            .map(GpuMaterial::from_material)
                            .collect::<Vec<_>>(),
                    ),
                    Some(&GpuLengthArray {
                        length: ArrayLength,
                        data: hyper_spheres,
                    }),
                    Some(&GpuLengthArray {
                        length: ArrayLength,
                        data: hyper_planes,
                    }),
                ),
            );
        }

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Rendering Encoder"),
        });
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Ray Tracing Pass"),
                    timestamp_writes: None,
                });

            compute_pass.set_pipeline(&self.ray_tracing_pipeline);
            compute_pass.set_bind_group(0, &self.ray_tracing_texture_output_bind_group, &[]);
            compute_pass.set_bind_group(1, self.camera_buffer.bind_group(), &[]);
            compute_pass.set_bind_group(2, self.objects_buffer.bind_group(), &[]);
            compute_pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
        }

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

            render_pass.set_pipeline(&self.ray_tracing_render_pipeline);
            render_pass.set_bind_group(0, &self.ray_tracing_texture_render_bind_group, &[]);
            render_pass.draw(0..4, 0..1);

            let info = GpuUiInfo {
                aspect: width as f32 / height as f32,
            };
            let mut lines = vec![
                GpuLine {
                    a: cgmath::vec2(0.02, 0.0),
                    b: cgmath::vec2(-0.02, 0.0),
                    width: 0.005,
                    color: cgmath::vec4(0.0, 0.0, 0.0, 1.0),
                },
                GpuLine {
                    a: cgmath::vec2(0.0, 0.02),
                    b: cgmath::vec2(0.0, -0.02),
                    width: 0.005,
                    color: cgmath::vec4(0.0, 0.0, 0.0, 1.0),
                },
            ];

            if let Some(index) = self.selected_hyper_sphere {
                let mut axis_lines =
                    Self::get_axis_lines(&self.camera, &self.objects[index], self.use_camera_axes);
                for (index, line) in axis_lines.iter_mut().enumerate() {
                    if let Some((line, _)) = line {
                        if self
                            .axis_line_interaction
                            .as_ref()
                            .is_some_and(|interaction| interaction.axis_index == index)
                        {
                            line.color *= 2.0;
                        }
                    }
                }
                axis_lines.sort_by(|a, b| match (a, b) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (Some((_, distance_a)), Some((_, distance_b))) => {
                        distance_a.total_cmp(distance_b).reverse()
                    }
                });
                lines.extend(axis_lines.into_iter().flatten().map(|(line, _)| line));
            }

            self.ui_buffer
                .write(device, queue, (Some(&info), Some(&lines)));

            render_pass.set_pipeline(&self.ui_render_pipeline);
            render_pass.set_bind_group(0, self.ui_buffer.bind_group(), &[]);
            render_pass.draw(
                0..4,
                0..lines
                    .len()
                    .try_into()
                    .expect("there should be less than u32::MAX lines"),
            );
        }
        command_encoder.copy_texture_to_texture(
            self.final_texture.as_image_copy(),
            texture.as_image_copy(),
            texture.size(),
        );

        queue.submit(std::iter::once(command_encoder.finish()));
    }
}

fn ray_tracing_texture_bind_group_layouts(
    device: &wgpu::Device,
) -> (wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
    let output_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Texture Output Bind Group Layout"),
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
            label: Some("Ray Tracing Texture Render Bind Group Layout"),
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

fn ray_tracing_texture_and_bind_groups(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    output_layout: &wgpu::BindGroupLayout,
    render_layout: &wgpu::BindGroupLayout,
) -> (wgpu::Texture, wgpu::BindGroup, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Ray Tracing Texture"),
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
        label: Some("Ray Tracing Texture Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let output_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Ray Tracing Texture Output Bind Group"),
        layout: output_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
    });
    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Ray Tracing Texture Render Bind Group"),
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
