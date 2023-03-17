use std::time::Instant;

use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub mod engine;
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
            msg!(error, err);
            panic!("{}", err);
        }
    };

    let mut start_time = Instant::now();
    let mut visible = true;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        Event::MainEventsCleared => {
            let delta_time = start_time.elapsed();

            if !visible {
                return;
            }

            if renderer.rebuild_swapchain {
                renderer.rebuild_swapchain = false;
                if let Err(msg) = renderer.resize(&window) {
                    msg!(error, msg);
                    return;
                }
            }

            if let Err(msg) = renderer.draw(&delta_time) {
                msg!(error, msg);
                *control_flow = ControlFlow::Exit;
                return;
            }
            start_time = Instant::now();
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(physical_size),
            ..
        } => {
            renderer.rebuild_swapchain = true;
            if physical_size.width < 10 || physical_size.height < 10 {
                visible = false;
                msg!(
                    info,
                    "Window is currently not visible, will not render anything"
                );
            } else {
                visible = true;
            }
        }
        _ => {}
    });
}
