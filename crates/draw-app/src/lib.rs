use std::net::SocketAddr;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

const DEFAULT_WIDTH: f64 = 1280.0;
const DEFAULT_HEIGHT: f64 = 800.0;

/// Set up a minimal macOS menu bar with a Quit item (Cmd+Q).
#[cfg(target_os = "macos")]
fn setup_macos_menu() {
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
    use objc2_foundation::NSString;

    // Safety: this runs inside tao's event loop which is always on the main thread
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);

    let menu_bar = NSMenu::new(mtm);

    // App menu (first item is the application menu on macOS)
    let app_menu_item = NSMenuItem::new(mtm);
    let app_menu = NSMenu::new(mtm);

    // Quit item with Cmd+Q
    let quit_title = NSString::from_str("Quit draw");
    let quit_key = NSString::from_str("q");
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &quit_title,
            Some(objc2::sel!(terminate:)),
            &quit_key,
        )
    };
    app_menu.addItem(&quit_item);
    app_menu_item.setSubmenu(Some(&app_menu));
    menu_bar.addItem(&app_menu_item);

    app.setMainMenu(Some(&menu_bar));
}

#[cfg(not(target_os = "macos"))]
fn setup_macos_menu() {}

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

    setup_macos_menu();

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
