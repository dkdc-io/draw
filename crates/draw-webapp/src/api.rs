use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use draw_core::document::Document;
use draw_core::storage;

#[derive(serde::Serialize)]
pub struct DrawingEntry {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub async fn list_drawings() -> impl IntoResponse {
    match storage::list_drawings() {
        Ok(drawings) => {
            let entries: Vec<DrawingEntry> = drawings
                .into_iter()
                .map(|(name, path)| {
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .strip_suffix(".draw")
                        .unwrap_or("")
                        .to_string();
                    DrawingEntry {
                        id,
                        name,
                        path: path.to_string_lossy().to_string(),
                    }
                })
                .collect();
            Json(entries).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn validate_id(id: &str) -> Result<(), StatusCode> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

pub async fn load_drawing(Path(id): Path<String>) -> impl IntoResponse {
    if let Err(status) = validate_id(&id) {
        return status.into_response();
    }
    let dir = match storage::storage_dir() {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let path = dir.join(format!("{id}.draw.json"));
    match storage::load(&path) {
        Ok(doc) => Json(doc).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "drawing not found").into_response(),
    }
}

pub async fn save_drawing(Json(doc): Json<Document>) -> impl IntoResponse {
    if let Err(status) = validate_id(&doc.id) {
        return status.into_response();
    }
    match storage::save_to_storage(&doc) {
        Ok(path) => Json(serde_json::json!({
            "id": doc.id,
            "path": path.to_string_lossy()
        }))
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_drawing(Path(id): Path<String>) -> impl IntoResponse {
    if let Err(status) = validate_id(&id) {
        return status.into_response();
    }
    let dir = match storage::storage_dir() {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let path = dir.join(format!("{id}.draw.json"));
    match std::fs::remove_file(&path) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn export_svg(Json(doc): Json<Document>) -> impl IntoResponse {
    let svg = draw_core::export_svg(&doc);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("image/svg+xml"),
    );
    (headers, svg)
}

pub async fn export_png(Json(doc): Json<Document>) -> impl IntoResponse {
    match draw_core::export_png(&doc) {
        Ok(png_bytes) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("image/png"),
            );
            (StatusCode::OK, headers, png_bytes).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
