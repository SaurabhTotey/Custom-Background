mod window;
use winit::event_loop::EventLoop;

fn main() {
	env_logger::init();

	// Get whether this demo should be run in the background, which is determined by whether the user specified it as a
	// command line argument. Default behaviour (for debugging purposes) is to assume the window won't be run in the
	// background.
	let args = std::env::args().collect::<Vec<_>>();
	let is_background_window = args.len() > 1 && args[1] == "background";

	// Create the window and let it run
	let event_loop = EventLoop::new();
	let demo_window =
		pollster::block_on(window::DemoWindow::new(&event_loop, is_background_window));
	demo_window.run(event_loop);
}
