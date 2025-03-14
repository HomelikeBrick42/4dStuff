use crate::gpu_buffers::Buffer;
use encase::{ShaderSize, StorageBuffer, internal::WriteInto};
use std::marker::PhantomData;

pub struct FixedSizeBuffer<T> {
    buffer: wgpu::Buffer,
    _data: PhantomData<T>,
}

impl<T: ShaderSize + WriteInto> FixedSizeBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &'static str,
        usage: wgpu::BufferUsages,
        data: &T,
    ) -> FixedSizeBuffer<T> {
        let mut this = Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(name),
                size: T::SHADER_SIZE.get(),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            _data: PhantomData,
        };
        this.write(device, queue, data);
        this
    }
}

impl<T: ShaderSize + WriteInto> Buffer for FixedSizeBuffer<T> {
    type Data = T;

    fn min_size() -> std::num::NonZero<wgpu::BufferAddress> {
        T::SHADER_SIZE
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn write(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, data: &Self::Data) -> bool {
        let mut buffer = queue
            .write_buffer_with(&self.buffer, 0, T::SHADER_SIZE)
            .expect("the buffer should be big enough to write the T");
        StorageBuffer::new(&mut *buffer)
            .write(data)
            .expect("the data should be successfully written");
        false
    }
}
