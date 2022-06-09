use crate::scene::bouncing_cube::BouncingCubeScene;
use crate::scene::Scene;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	//platform::unix::{WindowBuilderExtUnix, XWindowType},
	window::{Window, WindowBuilder},
};

pub struct DemoWindow {
	window: Window,
	window_size: winit::dpi::PhysicalSize<u32>,
	surface_configuration: wgpu::SurfaceConfiguration,
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	scene: Box<dyn Scene>,
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
				//.with_x11_window_type(vec![XWindowType::Desktop])
				//.with_override_redirect(true)
				.build(event_loop)
				.unwrap()
		} else {
			// Just create a window since it's not the background; this is for debugging purposes and making things
			// easier to run. Since I'm using a tiling window manager, I don't really care about the size.
			Window::new(event_loop).unwrap()
		};
		let window_size = window.inner_size();

		// Create surface.
		let instance = wgpu::Instance::new(wgpu::Backends::all());
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
					features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::DEPTH_CLIP_CONTROL,
					limits: wgpu::Limits {
						max_push_constant_size: 128,
						..wgpu::Limits::default()
					},
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

		// Make the scene
		let scene = Box::new(BouncingCubeScene::new(&device, &surface_configuration));

		Self {
			window,
			window_size,
			surface_configuration,
			surface,
			device,
			queue,
			scene,
		}
	}

	/**
	 * Handle updating this struct when the user requests a window resize.
	 */
	fn handle_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.surface_configuration.width = new_size.width;
		self.surface_configuration.height = new_size.height;
		self.surface
			.configure(&self.device, &self.surface_configuration);
		self.window_size = new_size;
		self.scene.resize(&self.device, &self.surface_configuration);
	}

	/**
	 * Draw a frame. Should only be called when redraws are requested from the window.
	 */
	fn draw_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let output_texture_view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let mut command_encoder =
			self.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("Default command encoder"),
				});
		self.scene
			.render(&mut command_encoder, &self.queue, &output_texture_view);
		self.queue.submit(std::iter::once(command_encoder.finish()));
		output.present();
		Ok(())
	}

	/**
	 * Consume the DemoWindow and EventLoop and run management on the window.
	 * While the window is open, this function is blocking.
	 */
	pub fn run(mut self, event_loop: EventLoop<()>) {
		let mut previous_frame_time = 0.0;
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
				Event::RedrawRequested(window_id) if window_id == self.window.id() => {
					let frame_start_instant = std::time::Instant::now();
					self.scene.update(previous_frame_time);
					let frame_draw_result = self.draw_frame();
					match frame_draw_result {
						Ok(_) => (),
						Err(wgpu::SurfaceError::Lost) => self.handle_resize(self.window_size),
						Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
						Err(_) => (),
					}
					previous_frame_time = frame_start_instant.elapsed().as_secs_f32();
				}
				_ => (),
			}
		});
	}
}
