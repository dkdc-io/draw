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
        .route("/style.css", get(serve_css))
        .route("/favicon.png", get(serve_favicon))
        .route("/theme.js", get(serve_theme_js))
        .route("/app.js", get(serve_app_js))
        .route("/canvas.js", get(serve_canvas_js))
        .route("/interactions.js", get(serve_interactions_js))
        .route("/api.js", get(serve_api_js))
        .route("/wasm_glue.js", get(serve_wasm_glue_js))
        .route("/draw_wasm_bg.wasm", get(serve_wasm_bg))
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
        format!(r#"<script>window.__OPEN_DRAWING_ID = "{id}";</script>"#)
    } else {
        String::new()
    };
    Html(INDEX_HTML.replace("<!-- OPEN_SCRIPT -->", &open_script))
}

async fn serve_css() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/css"));
    (headers, STYLE_CSS)
}

async fn serve_favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    (headers, FAVICON)
}
async fn serve_theme_js() -> impl IntoResponse {
    js_response(THEME_JS)
}
async fn serve_app_js() -> impl IntoResponse {
    js_response(APP_JS)
}
async fn serve_canvas_js() -> impl IntoResponse {
    js_response(CANVAS_JS)
}
async fn serve_interactions_js() -> impl IntoResponse {
    js_response(INTERACTIONS_JS)
}
async fn serve_api_js() -> impl IntoResponse {
    js_response(API_JS)
}
async fn serve_wasm_glue_js() -> impl IntoResponse {
    js_response(WASM_GLUE_JS)
}
async fn serve_wasm_bg() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/wasm"),
    );
    (headers, WASM_BG)
}

fn js_response(content: &'static str) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript"),
    );
    (headers, content)
}

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
