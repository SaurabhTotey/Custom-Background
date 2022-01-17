mod window;
mod scene;
use winit::event_loop::EventLoop;

fn main() {
	env_logger::init();

	// Get whether this demo should be run in the background, which is determined by whether this is built in release
	// mode or debug mode. The window is a background window in release mode.
	let is_background_window = !cfg!(debug_assertions);

	// Create the window and let it run
	let event_loop = EventLoop::new();
	let demo_window =
		pollster::block_on(window::DemoWindow::new(&event_loop, is_background_window));
	demo_window.run(event_loop);
}
