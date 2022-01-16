mod window;
use winit::event_loop::EventLoop;

fn main() {
	env_logger::init();

	let args = std::env::args().collect::<Vec<_>>();
	let is_background_window = args.len() > 1 && args[1] == "background";

	let event_loop = EventLoop::new();
	let demo_window =
		pollster::block_on(window::DemoWindow::new(&event_loop, is_background_window));
	demo_window.run(event_loop);
}
