use crate::gpu_buffers::Buffer;
use encase::{ShaderType, StorageBuffer, internal::WriteInto};
use std::marker::PhantomData;

pub struct DynamicBuffer<T: ?Sized> {
    name: &'static str,
    buffer: wgpu::Buffer,
    _data: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + ?Sized> DynamicBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &'static str,
        usage: wgpu::BufferUsages,
        data: &T,
    ) -> Self {
        let mut this = Self {
            name,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(name),
                size: data.size().get(),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            _data: PhantomData,
        };
        this.write(device, queue, data);
        this
    }
}

impl<T: ShaderType + WriteInto + ?Sized> Buffer for DynamicBuffer<T> {
    type Data = T;

    fn min_size() -> std::num::NonZero<wgpu::BufferAddress> {
        T::min_size()
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &Self::Data) -> bool {
        let new_size = data.size();
        let should_resize = new_size.get() > self.buffer.size();
        if should_resize {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.name),
                size: new_size.get(),
                usage: self.buffer.usage(),
                mapped_at_creation: false,
            });
        }

        let mut buffer = queue
            .write_buffer_with(&self.buffer, 0, new_size)
            .expect("the buffer should be big enough to write the T");
        StorageBuffer::new(&mut *buffer)
            .write(&data)
            .expect("the data should be successfully written");

        should_resize
    }
}
