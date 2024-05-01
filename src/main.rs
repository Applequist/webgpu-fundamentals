use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit::window::WindowId;
use webgpu_fundamentals::State;

#[derive(Default)]
pub struct App<'a> {
    windows: HashMap<WindowId, (Arc<Window>, State<'a>)>,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(event_loop
                .create_window(Window::default_attributes())
                .unwrap());
        let state = pollster::block_on(State::new(Arc::clone(&window)));
        self.windows.insert(window.id(), (window, state));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let (_window, state) = self.windows.get_mut(&window_id).unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping...");
                event_loop.exit();
            },
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            },
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.size());
                    },
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("Out of Memory Error: exiting...");
                        event_loop.exit();
                    },
                    Err(e) => eprint!("{:?}", e),
                }
            },
            _ => (),
        }
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

fn main() {
    run();
}
