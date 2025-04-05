use crate::camera::Camera;
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

const RENDER_SAMPLES: u32 = 1;

pub struct State {
    mouse_locked: bool,
    camera: Camera,

    final_texture: wgpu::Texture,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> State {
        let camera = Camera::default();
        let final_texture = final_texture(device, 1, 1);

        _ = queue;

        State {
            camera,
            mouse_locked: false,

            final_texture,
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
            texture.size(),
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
