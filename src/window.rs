use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::{WindowBuilderExtUnix, XWindowType},
	window::{Window, WindowBuilder},
};

pub struct DemoWindow {
	window: Window,
	surface_configuration: wgpu::SurfaceConfiguration,
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
}

impl DemoWindow {
	/**
	 * Create a new DemoWindow.
	 * Creating a background window assumes that X is being used.
	 */
	pub async fn new(event_loop: &EventLoop<()>, is_background: bool) -> Self {
		// Create the window.
		let window = if is_background {
			// Create a window for the background that isn't managed by window managers and that has the size of the
			// primary monitor.
			let monitor_size = event_loop.primary_monitor().unwrap().size();
			WindowBuilder::new()
				.with_inner_size(monitor_size)
				.with_x11_window_type(vec![XWindowType::Desktop])
				.with_override_redirect(true)
				.build(event_loop)
				.unwrap()
		} else {
			// Just create a window since it's not the background; this is for debugging purposes and making things
			// easier to run. Since I'm using a tiling window manager, I don't really care about the size.
			Window::new(event_loop).unwrap()
		};

		// Create surface.
		let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
		let surface = unsafe { instance.create_surface(&window) };

		// Create the device and the queue.
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				compatible_surface: Some(&surface),
			})
			.await
			.unwrap();
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: Some("Default device"),
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
				},
				None,
			)
			.await
			.unwrap();

		// Configure the surface.
		let surface_configuration = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
			width: window.inner_size().width,
			height: window.inner_size().height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &surface_configuration);

		return Self {
			window,
			surface_configuration,
			surface,
			device,
			queue,
		};
	}

	/**
	 * Handle updating this struct when the user requests a window resize.
	 */
	fn handle_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.surface_configuration.width = new_size.width;
		self.surface_configuration.height = new_size.height;
		self.surface
			.configure(&self.device, &self.surface_configuration);
	}

	/**
	 * Consume the DemoWindow and EventLoop and run management on the window.
	 * While the window is open, this function is blocking.
	 */
	pub fn run(mut self, event_loop: EventLoop<()>) {
		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Wait;
			match event {
				Event::WindowEvent {
					event: ref window_event,
					window_id,
				} if window_id == self.window.id() => match window_event {
					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
					WindowEvent::Resized(new_size) => self.handle_resize(*new_size),
					WindowEvent::ScaleFactorChanged {
						scale_factor: _,
						new_inner_size: new_size,
					} => self.handle_resize(**new_size),
					_ => (),
				},
				Event::MainEventsCleared => self.window.request_redraw(),
				Event::RedrawRequested(window_id) if window_id == self.window.id() => {}
				_ => (),
			}
		});
	}
}
