use eframe::wgpu;
use encase::{
    ArrayLength, ShaderSize, ShaderType, StorageBuffer, UniformBuffer, internal::WriteInto,
};
use std::marker::PhantomData;

pub struct GpuBuffer<T, const SLICE: bool> {
    label: &'static str,
    usage: wgpu::BufferUsages,
    buffer: wgpu::Buffer,
    _element: PhantomData<T>,
}

impl<T: ShaderSize + WriteInto> GpuBuffer<T, false> {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &'static str,
        mut usage: wgpu::BufferUsages,
        data: &T,
    ) -> Self {
        usage |= wgpu::BufferUsages::COPY_DST;
        let this = Self {
            label,
            usage,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: T::SHADER_SIZE.get(),
                usage,
                mapped_at_creation: false,
            }),
            _element: PhantomData,
        };
        this.write(queue, data);
        this
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        let mut bytes = queue
            .write_buffer_with(&self.buffer, 0, T::SHADER_SIZE)
            .expect("there should be enough space to write the data");

        if self.usage.contains(wgpu::BufferUsages::UNIFORM) {
            UniformBuffer::new(&mut *bytes)
                .write(&data)
                .expect("the bytes slice should be big enough to write the data");
        } else if self.usage.contains(wgpu::BufferUsages::STORAGE) {
            StorageBuffer::new(&mut *bytes)
                .write(&data)
                .expect("the bytes slice should be big enough to write the data");
        } else {
            panic!("unexpected buffer usage")
        }
    }
}

impl<T, const SLICE: bool> GpuBuffer<T, SLICE> {
    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn resize(&mut self, device: &wgpu::Device, new_size: wgpu::BufferAddress) {
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label),
            size: new_size,
            usage: self.usage,
            mapped_at_creation: false,
        });
    }
}

#[derive(ShaderType)]
struct GpuSlice<'a, T: ShaderSize + 'a> {
    length: ArrayLength,
    #[size(runtime)]
    data: &'a [T],
}

impl<T: ShaderSize + WriteInto> GpuBuffer<T, true> {
    pub fn new_slice(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &'static str,
        mut usage: wgpu::BufferUsages,
        data: &[T],
    ) -> Self {
        usage |= wgpu::BufferUsages::COPY_DST;
        let mut this = Self {
            label,
            usage,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: GpuSlice {
                    length: ArrayLength,
                    data,
                }
                .size()
                .get(),
                usage,
                mapped_at_creation: false,
            }),
            _element: PhantomData,
        };
        debug_assert!(!this.write(device, queue, data));
        this
    }

    /// Returns whether the buffer was resized
    #[must_use]
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) -> bool {
        let data = GpuSlice {
            length: ArrayLength,
            data,
        };

        let data_size = data.size();
        let should_resize = data_size.get() > self.buffer.size();

        if should_resize {
            self.resize(device, data_size.get());
        }

        let mut bytes = queue
            .write_buffer_with(&self.buffer, 0, data_size)
            .expect("there should be enough space to write the data");
        StorageBuffer::new(&mut *bytes)
            .write(&data)
            .expect("the bytes slice should be big enough to write the data");

        should_resize
    }
}
