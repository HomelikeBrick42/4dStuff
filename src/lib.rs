pub struct State {}

impl State {
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> State {
        State {}
    }

    pub fn update(&mut self, _delta_time: std::time::Duration) {}

    pub fn render(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _texture: &wgpu::Texture) {}
}
