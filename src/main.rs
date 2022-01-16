use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::WindowBuilderExtUnix,
	platform::unix::XWindowType,
	window::WindowBuilder,
};

fn main() {
	env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_x11_window_type(vec![XWindowType::Desktop])
		.build(&event_loop)
		.unwrap();

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
