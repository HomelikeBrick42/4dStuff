mod dynamic_buffer;
mod fixed_size_buffer;

pub use dynamic_buffer::*;
pub use fixed_size_buffer::*;

mod private {
    pub trait BufferTupleSealed {}
}
use private::BufferTupleSealed;

pub trait Buffer {
    type Data: ?Sized;
    fn min_size() -> std::num::NonZero<wgpu::BufferAddress>;
    fn buffer(&self) -> &wgpu::Buffer;
    /// Returns whether the buffer was recreated
    fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &Self::Data) -> bool;
}

pub trait BufferTuple: BufferTupleSealed {
    type Array<Item>: AsRef<[Item]>;
    type CreationInfo;
    type WriteInput<'a>
    where
        Self: 'a;

    fn construct(info: Self::CreationInfo) -> Self;
    fn bind_group_layout_entries(
        info: &Self::CreationInfo,
    ) -> Self::Array<wgpu::BindGroupLayoutEntry>;
    fn bind_group_entries(&self) -> Self::Array<wgpu::BindGroupEntry<'_>>;
    fn write<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        write_input: Self::WriteInput<'a>,
    ) -> bool
    where
        Self: 'a;
}

pub struct BufferCreationInfo<T> {
    pub buffer: T,
    pub binding_type: wgpu::BufferBindingType,
    pub visibility: wgpu::ShaderStages,
}

pub struct BufferGroup<G> {
    group_data: G,
    name: &'static str,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl<G: BufferTuple> BufferGroup<G> {
    pub fn new(device: &wgpu::Device, name: &'static str, info: G::CreationInfo) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(name),
            entries: G::bind_group_layout_entries(&info).as_ref(),
        });
        let group_data = G::construct(info);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &bind_group_layout,
            entries: G::bind_group_entries(&group_data).as_ref(),
        });
        BufferGroup {
            name,
            group_data,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        write_input: G::WriteInput<'_>,
    ) {
        if G::write(&mut self.group_data, device, queue, write_input) {
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(self.name),
                layout: &self.bind_group_layout,
                entries: G::bind_group_entries(&self.group_data).as_ref(),
            });
        }
    }
}

macro_rules! count_names {
    () => { 0 };
    ($first_name:ident $($rest:ident)*) => { 1 + count_names!($($rest)*) };
}

macro_rules! tuple_impls {
    ($($names:ident $second_names:ident),*) => {
        impl<$($names: Buffer,)*> BufferTupleSealed for ($($names,)*) {}

        #[allow(non_snake_case, unused)]
        impl<$($names: Buffer,)*> BufferTuple for ($($names,)*) {
            type Array<Elem> = [Elem; count_names!($($names)*)];
            type CreationInfo = ($(BufferCreationInfo<$names>,)*);
            type WriteInput<'a>
                = ($(Option<&'a $names::Data>,)*)
            where
                Self: 'a;

            fn construct(info: Self::CreationInfo) -> Self {
                let ($($names,)*) = info;
                #[allow(clippy::unused_unit)]
                ($($names.buffer,)*)
            }

            fn bind_group_layout_entries(
                info: &Self::CreationInfo,
            ) -> Self::Array<wgpu::BindGroupLayoutEntry> {
                let mut counter = 0u32;
                #[allow(unused)]
                let mut counter = move || {
                    let value = counter;
                    counter += 1;
                    value
                };

                let ($($names,)*) = info;
                [$(wgpu::BindGroupLayoutEntry {
                    binding: counter(),
                    visibility: $names.visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: $names.binding_type,
                        has_dynamic_offset: false,
                        min_binding_size: Some($names::min_size()),
                    },
                    count: None,
                },)*]
            }

            fn bind_group_entries(&self) -> Self::Array<wgpu::BindGroupEntry<'_>> {
                let mut counter = 0u32;
                #[allow(unused)]
                let mut counter = move || {
                    let value = counter;
                    counter += 1;
                    value
                };

                let ($($names,)*) = self;
                [$(wgpu::BindGroupEntry {
                    binding: counter(),
                    resource: $names.buffer().as_entire_binding(),
                },)*]
            }

            fn write<'a>(
                &mut self,
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                write_input: Self::WriteInput<'a>,
            ) -> bool
            where
                Self: 'a,
            {
                let ($($names,)*) = self;
                let ($($second_names,)*) = write_input;
                $($second_names.is_some_and(|input| {
                    $names.write(device, queue, input)
                }) |)* false
            }
        }
    };
}

tuple_impls!();
tuple_impls!(A B);
tuple_impls!(A B, C D);
tuple_impls!(A B, C D, E F);
tuple_impls!(A B, C D, E F, G H);
tuple_impls!(A B, C D, E F, G H, I J);
tuple_impls!(A B, C D, E F, G H, I J, K L);
tuple_impls!(A B, C D, E F, G H, I J, K L, M N);
tuple_impls!(A B, C D, E F, G H, I J, K L, M N, O P);
