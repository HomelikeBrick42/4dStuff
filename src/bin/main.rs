use std::sync::Arc;

use ray_tracer::State;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct App {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    state: State,
    window_state: Option<WindowState>,
    last_frame_time: Option<std::time::Instant>,
    delta_time: std::time::Duration,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_state = None;

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("4D Ray Tracer"))
                .expect("window should be successfully created"),
        );

        let size = window.inner_size();

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface should be created successfully");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&self.device, &surface_config);

        self.window_state = Some(WindowState {
            window,
            surface,
            surface_config,
        });
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        let time = std::time::Instant::now();
        self.delta_time = time.duration_since(self.last_frame_time.unwrap_or(time));
        self.last_frame_time = Some(time);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.state.update(self.delta_time);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_state = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let WindowState {
            window,
            surface,
            surface_config,
            ..
        } = self
            .window_state
            .as_mut()
            .expect("if there is a window event the window should have been created");
        assert_eq!(window.id(), id);

        let mut resized = |surface_config: &mut wgpu::SurfaceConfiguration,
                           size: winit::dpi::PhysicalSize<u32>| {
            surface_config.width = size.width.max(1);
            surface_config.height = size.height.max(1);
            surface.configure(&self.device, surface_config);
            self.state
                .resize(&self.device, surface_config.width, surface_config.height);
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => resized(surface_config, size),

            WindowEvent::RedrawRequested => {
                let surface_texture = loop {
                    match surface.get_current_texture() {
                        Ok(texture) => break texture,

                        Err(e @ wgpu::SurfaceError::Timeout) => {
                            eprintln!("WARNING: {e}");
                        }

                        Err(wgpu::SurfaceError::Outdated) => {
                            let size = window.inner_size();
                            resized(surface_config, size);
                        }

                        Err(wgpu::SurfaceError::Lost) => {
                            surface.configure(&self.device, surface_config);
                        }

                        Err(e @ (wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other)) => {
                            eprintln!("ERROR: {e}");
                            return;
                        }
                    }
                };

                self.state
                    .render(&self.device, &self.queue, &surface_texture.texture);

                window.pre_present_notify();
                surface_texture.present();
                window.request_redraw();
            }

            _ => {}
        }
    }
}

fn main() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("an adapter should have been requested successfully");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
    ))
    .expect("device should have been requested successfully");

    let state = State::new(&device, &queue);

    let event_loop = EventLoop::new().expect("the event loop should be created");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut App {
            instance,
            device,
            queue,
            state,
            window_state: None,
            last_frame_time: None,
            delta_time: std::time::Duration::ZERO,
        })
        .expect("the event loop should be started");
}
