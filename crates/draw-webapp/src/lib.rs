use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use std::net::SocketAddr;
use std::sync::Arc;

mod api;

const INDEX_HTML: &str = include_str!("../frontend/index.html");
const STYLE_CSS: &str = include_str!("../frontend/style.css");
const APP_JS: &str = include_str!("../frontend/app.js");
const CANVAS_JS: &str = include_str!("../frontend/canvas.js");
const INTERACTIONS_JS: &str = include_str!("../frontend/interactions.js");
const API_JS: &str = include_str!("../frontend/api.js");
const HELPERS_JS: &str = include_str!("../frontend/helpers.js");
const THEME_JS: &str = include_str!("../frontend/theme.js");
const FAVICON: &[u8] = include_bytes!("../frontend/favicon.png");
const WASM_GLUE_JS: &str = include_str!("../frontend/wasm_glue.js");
const WASM_BG: &[u8] = include_bytes!("../frontend/draw_wasm_bg.wasm");

const PORT: u16 = 1213;

struct AppState {
    open_id: Option<String>,
}

pub fn create_router(open_id: Option<String>) -> Router {
    let state = Arc::new(AppState { open_id });

    Router::new()
        .route("/", get(index))
        .route(
            "/style.css",
            get(|| async { static_response(STYLE_CSS.as_bytes(), "text/css") }),
        )
        .route(
            "/favicon.png",
            get(|| async { static_response(FAVICON, "image/png") }),
        )
        .route(
            "/helpers.js",
            get(|| async { static_response(HELPERS_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/theme.js",
            get(|| async { static_response(THEME_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/app.js",
            get(|| async { static_response(APP_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/canvas.js",
            get(|| async { static_response(CANVAS_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/interactions.js",
            get(|| async { static_response(INTERACTIONS_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/api.js",
            get(|| async { static_response(API_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/wasm_glue.js",
            get(|| async { static_response(WASM_GLUE_JS.as_bytes(), "application/javascript") }),
        )
        .route(
            "/draw_wasm_bg.wasm",
            get(|| async { static_response(WASM_BG, "application/wasm") }),
        )
        .route("/api/drawings", get(api::list_drawings))
        .route("/api/drawings", post(api::save_drawing))
        .route("/api/drawings/{id}", get(api::load_drawing))
        .route("/api/drawings/{id}", delete(api::delete_drawing))
        .route("/api/export/svg", post(api::export_svg))
        .route("/api/export/png", post(api::export_png))
        .with_state(state)
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let open_script = if let Some(id) = &state.open_id {
        // Validate ID to prevent injection (alphanumeric + hyphens only)
        if id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            format!(r#"<script>window.__OPEN_DRAWING_ID = "{id}";</script>"#)
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    Html(INDEX_HTML.replace("<!-- OPEN_SCRIPT -->", &open_script))
}

fn static_response(
    content: &'static [u8],
    content_type: &'static str,
) -> impl IntoResponse + use<> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    (headers, content)
}

/// Run the axum webapp, serving the embedded frontend on [`PORT`].
///
/// If `open_id` is set, the browser opens that drawing on launch.
///
/// # Errors
/// Returns an error if the tokio runtime cannot be built, the port is in use,
/// or the server fails while running.
pub fn run_webapp(open_id: Option<String>) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let port = PORT;
        let app = create_router(open_id);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let listener = tokio::net::TcpListener::bind(addr).await?;
        let actual_addr = listener.local_addr()?;

        println!("draw webapp: http://localhost:{}", actual_addr.port());
        let _ = open::that(format!("http://localhost:{}", actual_addr.port()));

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
                println!("\nshutting down...");
            })
            .await?;
        Ok(())
    })
}
