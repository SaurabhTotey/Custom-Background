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
	pub async fn new(event_loop: &EventLoop<()>, is_background: bool) -> Self {
		let window = if is_background {
			let monitor_size = event_loop.primary_monitor().unwrap().size();
			WindowBuilder::new()
				.with_inner_size(monitor_size)
				.with_x11_window_type(vec![XWindowType::Desktop])
				.with_override_redirect(true)
				.build(event_loop)
				.unwrap()
		} else {
			Window::new(event_loop).unwrap()
		};
		return Self { window };
	}

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
