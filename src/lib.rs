pub struct State {}

impl State {
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> State {
        State {}
    }

    pub fn update(&mut self, _delta_time: std::time::Duration) {}

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rendering Encoder"),
        });
        {
            let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&Default::default()),
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
        queue.submit(std::iter::once(command_encoder.finish()));
    }
}
