use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::{WindowBuilderExtUnix, XWindowType},
	window::{Window, WindowBuilder},
};

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	env_logger::init();

	let event_loop = EventLoop::new();
	let monitor_size = event_loop.primary_monitor().unwrap().size();
	let window = if args.len() > 1 && args[1] == "BACKGROUND" {
		WindowBuilder::new()
			.with_inner_size(monitor_size)
			.with_x11_window_type(vec![XWindowType::Desktop])
			.with_override_redirect(true)
			.build(&event_loop)
			.unwrap()
	} else {
		Window::new(&event_loop).unwrap()
	};

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				window_id,
			} if window_id == window.id() => *control_flow = ControlFlow::Exit,
			_ => (),
		}
	});
}
