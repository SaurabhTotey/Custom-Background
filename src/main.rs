use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::WindowBuilderExtUnix,
	platform::unix::WindowExtUnix,
	platform::unix::XWindowType,
	window::WindowBuilder,
};

fn main() {
	env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_x11_window_type(vec![XWindowType::Desktop])
		.with_override_redirect(true)
		.build(&event_loop)
		.unwrap();

	// TODO: figure out if the below code (from https://stackoverflow.com/questions/45527720/how-can-i-make-a-window-override-redirect-with-glutin)
	//  is necessary: we already have aoverride redirect
	let x_display = window.xlib_display().unwrap() as *mut winit::platform::unix::x11::ffi::Display;
	let x_window = window.xlib_window().unwrap() as winit::platform::unix::x11::ffi::XID;
	let x_connection = std::sync::Arc::into_raw(window.xlib_xconnection().unwrap());
	unsafe {
		((*x_connection).xlib.XUnmapWindow)(x_display, x_window);
		((*x_connection).xlib.XMapWindow)(x_display, x_window);
	}

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
