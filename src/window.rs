use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::{WindowBuilderExtUnix, XWindowType},
	window::{Window, WindowBuilder},
};

pub struct DemoWindow {
	window: Window,
}

impl DemoWindow {
	/**
	 * Create a new DemoWindow.
	 * Creating a background window assumes that X is being used.
	 */
	pub async fn new(event_loop: &EventLoop<()>, is_background: bool) -> Self {
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
		return Self { window };
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
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == self.window.id() => *control_flow = ControlFlow::Exit,
				_ => (),
			}
		});
	}
}
