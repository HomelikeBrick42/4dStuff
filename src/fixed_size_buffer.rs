use encase::{ShaderSize, UniformBuffer, internal::WriteInto};
use std::marker::PhantomData;

/// Use [`create_fixed_size_buffer`] to construct this
pub struct FixedSizeBuffer<T> {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    _data: PhantomData<T>,
}

#[macro_export]
#[doc(hidden)]
macro_rules! _create_fixed_size_buffer {
    ($device:expr, $queue:expr, $name:literal, $usage:expr, $binding_type:expr, $visibility:expr, $data:expr $(,)?) => {
        $crate::fixed_size_buffer::FixedSizeBuffer::_new(
            $device,
            $queue,
            ::core::concat!($name, " Buffer"),
            ::core::concat!($name, " Bind Group Layout"),
            ::core::concat!($name, " Bind Group"),
            $usage,
            $binding_type,
            $visibility,
            $data,
        )
    };
}
#[doc(inline)]
pub use _create_fixed_size_buffer as create_fixed_size_buffer;

impl<T> FixedSizeBuffer<T> {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<T: ShaderSize + WriteInto> FixedSizeBuffer<T> {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn _new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer_name: &'static str,
        bind_group_layout_name: &'static str,
        bind_group_name: &'static str,
        usage: wgpu::BufferUsages,
        binding_type: wgpu::BufferBindingType,
        visibility: wgpu::ShaderStages,
        data: &T,
    ) -> FixedSizeBuffer<T> {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(buffer_name),
            size: T::SHADER_SIZE.get(),
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bind_group_layout_name),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: binding_type,
                    has_dynamic_offset: false,
                    min_binding_size: Some(T::SHADER_SIZE),
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(bind_group_name),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let mut this = FixedSizeBuffer {
            buffer,
            bind_group_layout,
            bind_group,
            _data: PhantomData,
        };
        this.update(queue, data);
        this
    }

    pub fn update(&mut self, queue: &wgpu::Queue, data: &T) {
        let mut buffer = queue
            .write_buffer_with(&self.buffer, 0, T::SHADER_SIZE)
            .expect("the buffer should be big enough to write the T");
        UniformBuffer::new(&mut *buffer)
            .write(data)
            .expect("the data should be successfully written");
    }
}
