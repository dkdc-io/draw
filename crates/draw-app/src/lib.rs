use std::net::SocketAddr;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

const DEFAULT_WIDTH: f64 = 1280.0;
const DEFAULT_HEIGHT: f64 = 800.0;

pub fn run_app(open_id: Option<String>) -> anyhow::Result<()> {
    // Start the axum webapp on a random available port in a background thread
    let (port_tx, port_rx) = std::sync::mpsc::channel::<u16>();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let mut shutdown_tx = Some(shutdown_tx);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            let app = draw_webapp::create_router(open_id);
            let addr = SocketAddr::from(([127, 0, 0, 1], 0));
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("failed to bind");
            let actual_port = listener.local_addr().unwrap().port();
            port_tx.send(actual_port).ok();

            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();
        });
    });

    let port = port_rx.recv().expect("failed to receive server port");
    let url = format!("http://127.0.0.1:{port}");

    // Create native window and webview
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("draw")
        .with_inner_size(tao::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new().with_url(&url).build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            if let Some(tx) = shutdown_tx.take() {
                tx.send(()).ok();
            }
            *control_flow = ControlFlow::Exit;
        }
    });
}
