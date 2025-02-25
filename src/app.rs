use crate::{Camera, DrawUi, HyperSphere, gpu_buffer::GpuBuffer, rotor::Rotor};
use cgmath::InnerSpace;
use eframe::{egui, wgpu};
use encase::{ShaderSize, ShaderType};
use serde::{Deserialize, Serialize};

#[derive(ShaderType)]
struct GpuCamera {
    pub position: cgmath::Vector4<f32>,
    pub forward: cgmath::Vector4<f32>,
    pub right: cgmath::Vector4<f32>,
    pub up: cgmath::Vector4<f32>,
    pub sun_direction: cgmath::Vector4<f32>,
    pub sun_color: cgmath::Vector3<f32>,
    pub ambient_color: cgmath::Vector3<f32>,
    pub up_sky_color: cgmath::Vector3<f32>,
    pub down_sky_color: cgmath::Vector3<f32>,
    pub aspect: f32,
}

#[derive(ShaderType)]
struct GpuHyperSphere {
    pub position: cgmath::Vector4<f32>,
    pub color: cgmath::Vector3<f32>,
    pub radius: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    camera: Camera,

    sun_direction: cgmath::Vector4<f32>,
    sun_color: cgmath::Vector3<f32>,
    ambient_color: cgmath::Vector3<f32>,
    up_sky_color: cgmath::Vector3<f32>,
    down_sky_color: cgmath::Vector3<f32>,

    hyper_spheres: Vec<HyperSphere>,
    next_hyper_sphere_id: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            camera: Camera {
                position: cgmath::vec4(-3.0, 0.0, 0.0, 0.0),
                base_rotation: Rotor::IDENTITY,
                vertical_angle: 0.0,
                volume_view_enabled: false,
                volume_view_percentage: 0.0,
            },

            sun_direction: cgmath::vec4(-0.2, 0.1, 1.0, 0.0),
            sun_color: cgmath::vec3(0.9, 0.8, 0.7),
            ambient_color: cgmath::vec3(0.1, 0.1, 0.1),
            up_sky_color: cgmath::vec3(0.5, 0.5, 0.9),
            down_sky_color: cgmath::vec3(0.2, 0.2, 0.2),

            hyper_spheres: vec![],
            next_hyper_sphere_id: 0,
        }
    }
}

pub struct App {
    last_frame: Option<std::time::Instant>,

    state: State,

    egui_texture_bind_group_layout: wgpu::BindGroupLayout,
    egui_texture: wgpu::Texture,
    egui_texture_bind_group: wgpu::BindGroup,
    egui_texture_id: egui::TextureId,

    camera_buffer: GpuBuffer<GpuCamera, false>,
    camera_bind_group: wgpu::BindGroup,

    hyper_spheres_bind_group_layout: wgpu::BindGroupLayout,
    hyper_spheres_buffer: GpuBuffer<GpuHyperSphere, true>,
    hyper_spheres_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        let state: State = eframe::get_value(
            cc.storage.expect("there should be an eframe storage"),
            "State",
        )
        .unwrap_or_default();

        let eframe::egui_wgpu::RenderState {
            device,
            queue,
            renderer,
            ..
        } = cc
            .wgpu_render_state
            .as_ref()
            .expect("the wgpu renderer should be in use");

        let egui_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Egui Texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let (egui_texture, egui_texture_view, egui_texture_bind_group) =
            Self::egui_texture(device, 1, 1, &egui_texture_bind_group_layout);
        let egui_texture_id = renderer.write().register_native_texture(
            device,
            &egui_texture_view,
            wgpu::FilterMode::Nearest,
        );

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuCamera::SHADER_SIZE),
                    },
                    count: None,
                }],
            });
        let camera_buffer = GpuBuffer::new(
            device,
            queue,
            "Camera Buffer",
            wgpu::BufferUsages::UNIFORM,
            &Self::get_gpu_camera(&state, 1.0),
        );
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.get_buffer().as_entire_binding(),
            }],
        });

        let hyper_spheres_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Hyper Spheres Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let hyper_spheres_buffer = GpuBuffer::new_slice(
            device,
            queue,
            "Hyper Spheres Buffer",
            wgpu::BufferUsages::STORAGE,
            &state
                .hyper_spheres
                .iter()
                .map(Self::hyper_sphere_to_gpu)
                .collect::<Vec<_>>(),
        );
        let hyper_spheres_bind_group = Self::hyper_spheres_bind_group(
            device,
            &hyper_spheres_buffer,
            &hyper_spheres_bind_group_layout,
        );

        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &egui_texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &hyper_spheres_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let ray_tracing_shader =
            device.create_shader_module(wgpu::include_wgsl!("./ray_tracing.wgsl"));
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Ok(Self {
            last_frame: None,

            state,

            egui_texture_bind_group_layout,
            egui_texture,
            egui_texture_bind_group,
            egui_texture_id,

            camera_buffer,
            camera_bind_group,

            hyper_spheres_bind_group_layout,
            hyper_spheres_buffer,
            hyper_spheres_bind_group,

            ray_tracing_pipeline,
        })
    }

    fn egui_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        egui_texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        let egui_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Egui Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let egui_texture_view = egui_texture.create_view(&Default::default());
        let egui_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Egui Texture Bind Group"),
            layout: egui_texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&egui_texture_view),
            }],
        });
        (egui_texture, egui_texture_view, egui_texture_bind_group)
    }

    fn hyper_spheres_bind_group(
        device: &wgpu::Device,
        hyper_spheres_buffer: &GpuBuffer<GpuHyperSphere, true>,
        hyper_spheres_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hyper Spheres Bind Group"),
            layout: hyper_spheres_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: hyper_spheres_buffer.get_buffer().as_entire_binding(),
            }],
        })
    }

    fn resize(&mut self, width: u32, height: u32, render_state: &eframe::egui_wgpu::RenderState) {
        let eframe::egui_wgpu::RenderState {
            device, renderer, ..
        } = render_state;

        let egui_texture_view;
        (
            self.egui_texture,
            egui_texture_view,
            self.egui_texture_bind_group,
        ) = Self::egui_texture(
            device,
            width.max(1),
            height.max(1),
            &self.egui_texture_bind_group_layout,
        );
        renderer.write().update_egui_texture_from_wgpu_texture(
            &render_state.device,
            &egui_texture_view,
            wgpu::FilterMode::Nearest,
            self.egui_texture_id,
        );
    }

    fn get_gpu_camera(state: &State, aspect: f32) -> GpuCamera {
        let rotation = state.camera.get_rotation();
        GpuCamera {
            position: state.camera.position,
            forward: rotation.rotate(cgmath::vec4(1.0, 0.0, 0.0, 0.0)),
            right: rotation.rotate(cgmath::vec4(0.0, 1.0, 0.0, 0.0)),
            up: rotation.rotate(cgmath::vec4(0.0, 0.0, 1.0, 0.0)),
            sun_direction: state.sun_direction.normalize(),
            sun_color: state.sun_color,
            ambient_color: state.ambient_color,
            up_sky_color: state.up_sky_color,
            down_sky_color: state.down_sky_color,
            aspect,
        }
    }

    fn hyper_sphere_to_gpu(hyper_sphere: &HyperSphere) -> GpuHyperSphere {
        let HyperSphere {
            name: _,
            ui_id: _,
            position,
            radius,
            color,
        } = *hyper_sphere;
        GpuHyperSphere {
            position,
            color,
            radius,
        }
    }

    fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (render_width, render_height) = (self.egui_texture.width(), self.egui_texture.height());

        let mut compute_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });
        {
            let mut ray_tracing_pass =
                compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Ray Tracing Pass"),
                    timestamp_writes: None,
                });

            ray_tracing_pass.set_pipeline(&self.ray_tracing_pipeline);
            ray_tracing_pass.set_bind_group(0, &self.egui_texture_bind_group, &[]);
            ray_tracing_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            ray_tracing_pass.set_bind_group(2, &self.hyper_spheres_bind_group, &[]);

            let workgroup_size = 16;
            ray_tracing_pass.dispatch_workgroups(
                render_width.div_ceil(workgroup_size),
                render_height.div_ceil(workgroup_size),
                1,
            );
        }
        queue.submit(std::iter::once(compute_encoder.finish()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let time = std::time::Instant::now();
        let dt = time.duration_since(self.last_frame.unwrap_or(time));
        self.last_frame = Some(time);

        let ts = dt.as_secs_f32();

        let mut camera_changed = false;
        let mut hyper_spheres_changed = false;

        egui::Window::new("Settings").show(ctx, |ui| {
            ui.label(format!("Frame Time: {:.2}ms", dt.as_secs_f64() * 1000.0));
            ui.label(format!("FPS: {:.2}", 1.0 / dt.as_secs_f64()));
            ui.collapsing("Camera", |ui| {
                camera_changed |= self.state.camera.draw_ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Sun Direction: ");
                camera_changed |= self.state.sun_direction.draw_ui(ui);
            });
            if ui.button("Normalise Sun Direction").clicked() {
                self.state.sun_direction = self.state.sun_direction.normalize();
                camera_changed = true;
            }
            ui.horizontal(|ui| {
                ui.label("Sun Color: ");
                camera_changed |= ui
                    .color_edit_button_rgb(self.state.sun_color.as_mut())
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("Ambient Color: ");
                camera_changed |= ui
                    .color_edit_button_rgb(self.state.ambient_color.as_mut())
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("Up Sky Color: ");
                camera_changed |= ui
                    .color_edit_button_rgb(self.state.up_sky_color.as_mut())
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("Down Sky Color: ");
                camera_changed |= ui
                    .color_edit_button_rgb(self.state.down_sky_color.as_mut())
                    .changed();
            });
            if ui.button("RESET SCENE").clicked() {
                self.state = State::default();
                camera_changed = true;
                hyper_spheres_changed = true;
            }
        });

        egui::Window::new("Hyper Spheres").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.state.hyper_spheres.retain_mut(|hyper_sphere| {
                    let mut deleted = false;
                    egui::CollapsingHeader::new(&hyper_sphere.name)
                        .id_salt(hyper_sphere.ui_id)
                        .show(ui, |ui| {
                            hyper_spheres_changed |= hyper_sphere.draw_ui(ui);
                            if ui.button("Delete").clicked() {
                                hyper_spheres_changed = true;
                                deleted = true;
                            }
                        });
                    !deleted
                });
                if ui.button("Add Hyper Sphere").clicked() {
                    let id = self.state.next_hyper_sphere_id;
                    self.state.next_hyper_sphere_id += 1;
                    self.state.hyper_spheres.push(HyperSphere {
                        name: "Default Hyper Sphere".into(),
                        ui_id: id,
                        position: cgmath::vec4(0.0, 0.0, 0.0, 0.0),
                        radius: 1.0,
                        color: cgmath::vec3(1.0, 1.0, 1.0),
                    });
                    hyper_spheres_changed = true;
                }
            });
        });

        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                let movement_amount = 4.0 * ts;
                let rotation_amount = std::f32::consts::FRAC_PI_2 * ts;

                let rotation = self.state.camera.get_rotation();
                let forward = rotation.rotate(cgmath::vec4(movement_amount, 0.0, 0.0, 0.0));
                let right = rotation.rotate(cgmath::vec4(0.0, movement_amount, 0.0, 0.0));
                let up = rotation.rotate(cgmath::vec4(0.0, 0.0, movement_amount, 0.0));
                let ana = rotation.rotate(cgmath::vec4(0.0, 0.0, 0.0, movement_amount));

                if i.key_down(egui::Key::W) {
                    self.state.camera.position += forward;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::S) {
                    self.state.camera.position -= forward;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::A) {
                    self.state.camera.position -= right;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::D) {
                    self.state.camera.position += right;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::Q) {
                    self.state.camera.position -= up;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::E) {
                    self.state.camera.position += up;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::R) {
                    self.state.camera.position += ana;
                    camera_changed = true;
                }
                if i.key_down(egui::Key::F) {
                    self.state.camera.position -= ana;
                    camera_changed = true;
                }

                #[expect(clippy::collapsible_else_if)]
                if !self.state.camera.volume_view_enabled {
                    if i.modifiers.shift {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xw(rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xw(-rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_yw(-rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_yw(rotation_amount);
                            camera_changed = true;
                        }
                    } else {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.state.camera.vertical_angle += rotation_amount;
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.state.camera.vertical_angle -= rotation_amount;
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xy(-rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xy(rotation_amount);
                            camera_changed = true;
                        }
                    }
                } else {
                    if i.modifiers.shift {
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_yw(rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_yw(-rotation_amount);
                            camera_changed = true;
                        }
                    } else {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xw(rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xw(-rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xy(-rotation_amount);
                            camera_changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.state.camera.base_rotation = self.state.camera.base_rotation
                                * Rotor::rotation_xy(rotation_amount);
                            camera_changed = true;
                        }
                    }
                }
            });
        }

        if camera_changed {
            self.state.camera.base_rotation = self.state.camera.base_rotation.normalized();
        }

        let volume_view_duration = 0.5;
        if self.state.camera.volume_view_enabled && self.state.camera.volume_view_percentage < 1.0 {
            self.state.camera.volume_view_percentage =
                (self.state.camera.volume_view_percentage + ts / volume_view_duration).min(1.0);
            if self.state.camera.volume_view_percentage == 1.0 {
                self.state.camera.vertical_angle = 0.0;
            }
            camera_changed = true;
        } else if !self.state.camera.volume_view_enabled
            && self.state.camera.volume_view_percentage > 0.0
        {
            self.state.camera.volume_view_percentage =
                (self.state.camera.volume_view_percentage - ts / volume_view_duration).max(0.0);
            camera_changed = true;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(255, 0, 255)))
            .show(ctx, |ui| {
                let render_state @ eframe::egui_wgpu::RenderState { device, queue, .. } = frame
                    .wgpu_render_state()
                    .expect("the wgpu renderer should be in use");

                let (_, rect) = ui.allocate_space(ui.available_size());
                let (width, height) = (rect.width() as u32, rect.height() as u32);

                if self.egui_texture.width() != width && self.egui_texture.height() != height {
                    self.resize(width, height, render_state);
                    camera_changed = true;
                }

                if camera_changed {
                    self.camera_buffer.write(
                        queue,
                        &Self::get_gpu_camera(&self.state, width as f32 / height as f32),
                    );
                }

                if hyper_spheres_changed {
                    let resized = self.hyper_spheres_buffer.write(
                        device,
                        queue,
                        &self
                            .state
                            .hyper_spheres
                            .iter()
                            .map(Self::hyper_sphere_to_gpu)
                            .collect::<Vec<_>>(),
                    );
                    if resized {
                        self.hyper_spheres_bind_group = Self::hyper_spheres_bind_group(
                            device,
                            &self.hyper_spheres_buffer,
                            &self.hyper_spheres_bind_group_layout,
                        );
                    }
                }

                self.render(device, queue);

                ui.painter().image(
                    self.egui_texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "State", &self.state)
    }
}
