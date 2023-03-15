use renderer::{utils::MAX_FRAME_DRAWS, Renderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

mod renderer;

fn main() {
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![simplelog::TermLogger::new(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )];

    if let Ok(file) = std::fs::File::create("log.txt") {
        loggers.push(simplelog::WriteLogger::new(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            file,
        ));
    }

    simplelog::CombinedLogger::init(loggers).unwrap();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("HAHA")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let mut renderer = match Renderer::new(&window) {
        Ok(base) => base,
        Err(err) => {
            debug_message!(error, err);
            panic!("{}", err);
        }
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                if window.inner_size().width == 0 && window.inner_size().height == 0 {
                    return;
                }

                if renderer.rebuild_swapchain {
                    renderer.rebuild_swapchain = false;

                    if let Err(msg) = renderer.base.resize(&window) {
                        debug_message!(error, msg);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if let Err(msg) = renderer.data.resize(&renderer.base) {
                        debug_message!(error, msg);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                if let Err(msg) = renderer.draw() {
                    debug_message!(error, msg);
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                renderer.current_frame_index = (renderer.current_frame_index + 1) % MAX_FRAME_DRAWS;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(..),
                ..
            } => {
                renderer.rebuild_swapchain = true;
            }
            _ => {}
        }
    });
}
