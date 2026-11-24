use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;

struct MyApp {
    ctx: palgl::PalCtx,
    rpass_builder: palgl::RenderPassBuilder
}

impl MyApp {
    fn new(window: &winit::window::Window) -> Self {
        let raw_display_handle = Some(window.display_handle().unwrap().as_raw().into());
        let raw_window_handle = window.window_handle().unwrap().as_raw().into();

        let conf = palgl::PalCtxConfig {
            init_width: window.inner_size().width,
            init_height: window.inner_size().height,
            ..Default::default()
        };

        let ctx = unsafe { palgl::new_palgl_ctx(palgl::InitContext { raw_display_handle, raw_window_handle }, &conf) };
        let rpass_builder = palgl::RenderPassBuilder::new();

        Self {
            ctx,
            rpass_builder
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize_surface(width, height);
    }

    fn draw(&mut self) {
        self.ctx.commit_frame();
    }
}

#[derive(Default)]
struct WinitApp {
    window: Option<winit::window::Window>,
    app: Option<MyApp>
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(winit::window::WindowAttributes::default()).unwrap();
        self.app = Some(MyApp::new(&window));
        self.window = Some(window);
    }
    
    fn window_event(
        &mut self,
        eloop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    )
    {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(app) = self.app.as_mut() {
                    app.draw();
                }
            },
            winit::event::WindowEvent::Resized(new_size) => {
                if let Some(app) = self.app.as_mut() {
                    app.resize(new_size.width, new_size.height);
                }
            }
            winit::event::WindowEvent::CloseRequested => eloop.exit(),
            _ => {}
        }
    }
}

fn main() {
    let eloop = winit::event_loop::EventLoopBuilder::default()
        .build()
        .unwrap();

    let _ = eloop.run_app(&mut WinitApp::default());
}