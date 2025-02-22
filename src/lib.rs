#![deny(rust_2018_idioms)]

use eframe::{egui, wgpu};
use encase::{ShaderSize, ShaderType, UniformBuffer};

#[derive(ShaderType)]
struct GpuCamera {
    pub position: cgmath::Vector4<f32>,
    pub forward: cgmath::Vector4<f32>,
    pub right: cgmath::Vector4<f32>,
    pub up: cgmath::Vector4<f32>,
    pub aspect: f32,
}

pub struct App {
    egui_texture: wgpu::Texture,
    egui_texture_bind_group_layout: wgpu::BindGroupLayout,
    egui_texture_bind_group: wgpu::BindGroup,
    egui_texture_id: egui::TextureId,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        let eframe::egui_wgpu::RenderState {
            device, renderer, ..
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
        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: GpuCamera::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&egui_texture_bind_group_layout, &camera_bind_group_layout],
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
            egui_texture,
            egui_texture_bind_group_layout,
            egui_texture_bind_group,
            egui_texture_id,
            camera_uniform_buffer,
            camera_bind_group,
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

    fn update_camera(&self, queue: &wgpu::Queue) {
        let (render_width, render_height) = (self.egui_texture.width(), self.egui_texture.height());

        let mut buffer = queue
            .write_buffer_with(&self.camera_uniform_buffer, 0, GpuCamera::SHADER_SIZE)
            .expect("the camera uniform buffer should be big enough to write a GpuCamera");
        UniformBuffer::new(&mut *buffer)
            .write(&GpuCamera {
                aspect: render_width as f32 / render_height as f32,
                position: cgmath::vec4(0.0, 0.0, 0.0, 0.0),
                forward: cgmath::vec4(1.0, 0.0, 0.0, 0.0),
                right: cgmath::vec4(0.0, 1.0, 0.0, 0.0),
                up: cgmath::vec4(0.0, 0.0, 1.0, 0.0),
            })
            .expect("the buffer should be big enough to write a GpuCamera");
    }

    fn render(&self, render_state: &eframe::egui_wgpu::RenderState) {
        let eframe::egui_wgpu::RenderState { device, queue, .. } = render_state;
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

            let workgroup_size = 16;
            ray_tracing_pass.dispatch_workgroups(
                render_width.div_ceil(workgroup_size),
                render_height.div_ceil(workgroup_size),
                1,
            );
        }
        queue.submit(Some(compute_encoder.finish()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(255, 0, 255)))
            .show(ctx, |ui| {
                let render_state = frame
                    .wgpu_render_state()
                    .expect("the wgpu renderer should be in use");

                let (_, rect) = ui.allocate_space(ui.available_size());
                let (width, height) = (rect.width() as u32, rect.height() as u32);

                if self.egui_texture.width() != width && self.egui_texture.height() != height {
                    self.resize(width, height, render_state);
                }

                self.update_camera(&render_state.queue);
                self.render(render_state);

                ui.painter().image(
                    self.egui_texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });

        ctx.request_repaint();
    }
}
